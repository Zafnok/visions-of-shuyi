"""The voice readers that `cargo xtask voice generate` runs (ticket 0721).

Each command runs in one of the voice lab's Python environments (ADR-0107
rule 3) and reads a job file the tool writes:

    {"seed": 7, "language": "en", "out": "<folder>",
     "lines": [{"key": "...", "text": "...", "direction": null,
                "reference": "<voice-refs clip>", "reference_text": "..."}]}

Commands (environment in brackets):

    qwen3 <job>     [qwen]     the Base weights read each line in the voice of
                               its reference clip: <out>/<key>.wav
    chatterbox <job> [chatter] Chatterbox reads each line from its reference
                               clip: <out>/<key>.wav
    directed <job>  [qwen]     ADR-0109's take: the VoiceDesign weights with the
                               direction in the instruct turn and the reference
                               clip's voice print: <out>/<key>.take.wav
    convert <job>   [chatter]  each take converted to its reference clip's
                               voice: <out>/<key>.take.wav -> <out>/<key>.wav
    hear <job>      [qwen]     faster-whisper's transcript of <out>/<key>.wav
                               for every line: <out>/heard.json, key -> words

Nothing here decides anything: which lines, which seed, and whether a take
says the right words are the tool's.
"""
import json
import os
import sys

QWEN_BASE = "Qwen/Qwen3-TTS-12Hz-1.7B-Base"
QWEN_DESIGN = "Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign"
LANGUAGES = {"en": "English", "ja": "Japanese", "zh": "Chinese"}
# ADR-0107 rule 2 and ADR-0109's recipe.
TEMPERATURE = 0.75
CHATTERBOX_EXAGGERATION = 0.5
CHATTERBOX_CFG = 0.4
WHISPER = {"en": "base.en"}


def qwen(name):
    import torch
    from qwen_tts import Qwen3TTSModel
    return Qwen3TTSModel.from_pretrained(name, device_map="cuda:0", dtype=torch.bfloat16, attn_implementation="sdpa")


def write(path, wav, sr):
    import soundfile as sf
    sf.write(path, wav, sr)


def qwen3(job):
    import torch
    model = qwen(QWEN_BASE)
    prompts = {}
    for line in job["lines"]:
        ref = line["reference"]
        if ref not in prompts:
            prompts[ref] = model.create_voice_clone_prompt(
                ref_audio=ref, ref_text=line["reference_text"], x_vector_only_mode=False)
        torch.manual_seed(job["seed"])
        wavs, sr = model.generate_voice_clone(
            text=line["text"], language=LANGUAGES[job["language"]], voice_clone_prompt=prompts[ref])
        write(os.path.join(job["out"], line["key"] + ".wav"), wavs[0], sr)
        print(line["key"], flush=True)


def chatterbox(job):
    import torch
    from chatterbox.mtl_tts import ChatterboxMultilingualTTS
    model = ChatterboxMultilingualTTS.from_pretrained(device="cuda", t3_model="v3")
    for line in job["lines"]:
        torch.manual_seed(job["seed"])
        wav = model.generate(line["text"], language_id=job["language"], audio_prompt_path=line["reference"],
                             exaggeration=CHATTERBOX_EXAGGERATION, cfg_weight=CHATTERBOX_CFG)
        write(os.path.join(job["out"], line["key"] + ".wav"), wav.squeeze().cpu().numpy(), model.sr)
        print(line["key"], flush=True)


def directed(job):
    import torch
    base = qwen(QWEN_BASE)
    prints = {line["reference"]: base.create_voice_clone_prompt(ref_audio=line["reference"], x_vector_only_mode=True)
              for line in job["lines"]}
    del base
    torch.cuda.empty_cache()
    m = qwen(QWEN_DESIGN)
    for line in job["lines"]:
        # No public method takes an instruct turn with a voice print: these
        # are the calls ADR-0109's listening rounds ran.
        ids = m._tokenize_texts([m._build_assistant_text(line["text"])])
        instruct = [m._tokenize_texts([m._build_instruct_text(line["direction"])])[0]]
        voice = m._prompt_items_to_voice_clone_prompt(prints[line["reference"]])
        torch.manual_seed(job["seed"])
        codes, _ = m.model.generate(
            input_ids=ids, ref_ids=[None], instruct_ids=instruct, voice_clone_prompt=voice,
            languages=[LANGUAGES[job["language"]]], non_streaming_mode=False,
            **m._merge_generate_kwargs(temperature=TEMPERATURE))
        wavs, sr = m.model.speech_tokenizer.decode([{"audio_codes": codes[0]}])
        write(os.path.join(job["out"], line["key"] + ".take.wav"), wavs[0], sr)
        print(line["key"], flush=True)


def convert(job):
    from chatterbox.mtl_tts import ChatterboxMultilingualTTS
    from chatterbox.vc import ChatterboxVC
    model = ChatterboxMultilingualTTS.from_pretrained(device="cuda", t3_model="v3")
    vc = ChatterboxVC(model.s3gen, "cuda")
    for line in job["lines"]:
        take = os.path.join(job["out"], line["key"] + ".take.wav")
        wav = vc.generate(take, target_voice_path=line["reference"])
        write(os.path.join(job["out"], line["key"] + ".wav"), wav.squeeze().cpu().numpy(), vc.sr)
        print(line["key"], flush=True)


def hear(job):
    # faster-whisper's CUDA libraries are the ones PyTorch ships (cuBLAS,
    # cuDNN); Windows finds them only on the PATH.
    import torch
    torch_lib = os.path.join(os.path.dirname(torch.__file__), "lib")
    os.environ["PATH"] = torch_lib + os.pathsep + os.environ["PATH"]
    from faster_whisper import WhisperModel
    lab = os.environ["VOICE_LAB"]
    model = WhisperModel(WHISPER.get(job["language"], "base"), device="cuda", compute_type="float16",
                         download_root=os.path.join(lab, "models", "whisper"))
    heard = {}
    for line in job["lines"]:
        segments, _ = model.transcribe(os.path.join(job["out"], line["key"] + ".wav"),
                                       language=job["language"], beam_size=5)
        heard[line["key"]] = " ".join(s.text for s in segments).strip()
    with open(os.path.join(job["out"], "heard.json"), "w", encoding="utf-8") as f:
        json.dump(heard, f, ensure_ascii=False)


COMMANDS = {"qwen3": qwen3, "chatterbox": chatterbox, "directed": directed, "convert": convert, "hear": hear}

if __name__ == "__main__":
    if len(sys.argv) != 3 or sys.argv[1] not in COMMANDS:
        sys.exit("usage: lab.py <" + "|".join(COMMANDS) + "> <job.json>")
    with open(sys.argv[2], encoding="utf-8") as f:
        COMMANDS[sys.argv[1]](json.load(f))
