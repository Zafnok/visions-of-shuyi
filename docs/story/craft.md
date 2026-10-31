# Craft: what makes fantasy game dialogue compelling

Research notes from ticket 0736 (2026-10-23). The rules they led to are the
**Craft** section of the `story-writing` skill
([`.claude/skills/story-writing/SKILL.md`](../../.claude/skills/story-writing/SKILL.md));
this file is the why, with the sources. Read "What went wrong" and "Why AI
dialogue reads bland" once; the rest is reference.

Nick, Chapter 1 playtest round 3 (0804, 2026-10-16), on the script after
two rewrites:

> "Fable still did it line by line. Still 14 archers stuff... We need to
> toss the script. We can keep the beats we agreed on but the whole script
> is fairly mid imo. We need to have you get a crash course on fantasy
> writing that's compelling. Idk a skill repo for writing? Your writing is
> so bland."

And on where the chapter drags: "In the conversations". Round 2, about
0724's rewrite: "It took the lines line by line and sucked out the quips
rather than reworking the script... this is just matter of fact and
weird."

Then, on 0736's first before/after samples (2026-10-23):

> "for the characters to have any emotional ties, for the deaths to
> matter, they need to be first and foremost engaging and emotional. These
> people are so clearly robots (i.e. Claude's specific vision of an
> optimized utopian world). Have more warmth without cringey jokes every 2
> lines. Just talk more human, I don't know. Run more research subagents
> with this in mind. Or if there is a better model more suited to writing
> that isn't so 'tell, don't show' the way you're writing now then find
> that model."

How the research was done, in two rounds. **Round 1** studied craft: 14
research agents (web: published writing skills, AI-writing guides, craft
books and talks, seven tactics games, two groups of novels, Nick's vibe
games, scene lengths, and studies of why model fiction reads bland), one
critic reading `ch01.dlg` and the two rewrites' notes, then three writers
and two blind judges for the first samples. Nick rejected those samples
as robots. **Round 2** studied warmth: six agents (how games make players
care before a death, warmth without jokes, how real people talk, why AI
characters read as robots, soldiers who love each other in novels and TV,
emotional scenes in tactics RPGs), one on which model writes the most
human fiction, and a critic on why the samples read as robots; then four
writers (two models, with and without a character method) and two blind
judges for new samples. Several agents read full fan transcripts (Fire
Emblem Wiki, Serenes Forest, the Final Fantasy Wiki, fedatamine) and
counted lines with scripts of their own. The shared web-search budget ran
out partway through both rounds, so some agents worked from direct page
fetches; anything they could only recall is left out here or marked.

**Round 1's rules overcorrected.** "Nobody says the feeling", "every box
is someone doing something to someone" and "facts are ammunition" each
came from good advice against reports, and together they allowed only one
kind of person. The round 1 findings below are still what those games do;
the skill now weighs them under round 2's.

## What went wrong in 0724 and 0728

### The process

- **Both rewrites edited the script; neither threw it away.** 0724's step
  was "for each joke, decide: cut, turn into a plain line that carries the
  same information, or keep". A plain line that carries the same
  information is a report, and that is what Nick got. 0728 was asked for
  scenes, and its own notes say the scene shapes "did not move" and it
  "did not merge or reorder boxes between speakers". Both shipped a table
  of old and new lines.
- **The facts were fixed, so voice was the only thing left to change.**
  Every story fact, reply text and ledger entry was held, and Aske's count
  was tied to the map by a scope line. Her fort line has been rewritten
  three times around the same number.
- **The "angle on a fact" rule kept the fact in the middle.** 0728 was told
  to give each fact its speaker's angle, and added detail to the report
  ("I counted at dusk and again at first light") instead of asking whether
  the scene needed a count at all.
- **The fixes made it longer.** Opening plus the green went from 38–39
  boxes to 39–41 while Nick was saying the chapter drags.
- **The reference files now teach the rejected lines.** 0724 gave every
  character sheet the same "under threat" mode (short, exact, says the
  plain thing), and all of Chapter 1 is under threat. 0724 and 0728 put the
  new script's lines into the sheets as sample lines: Aske's sample line is
  the rejected fort line, word for word, and her voice note says she "says
  what she counted". The beat sheet quotes lines too, and its **Wants**
  lines were written after reading the old script, so they describe it.
  Two of the guides below say examples steer a model harder than any
  rule; the writer did what the files showed. (The skill's blank-page
  rule now says to skip quoted lines; 0737 rewrites the sheets' voice
  notes.)
- **The critique pass graded itself.** 0728 ticked every box of the
  checklist, judging "a line another character could say" by what each
  person notices rather than how they talk. Nothing in the checklist
  tested for a report, for one rhythm in every voice, or for length.

### What the script does

The critic counted and quoted `assets/dialogue/ch01.dlg` (as of `main` on
2026-10-23):

| Scene | Boxes (longest path) | Note |
| ----- | -------------------- | ---- |
| `ch01_intro` | 19 | the letter is read at box 11; four boxes of wind-down after the choice |
| `ch01_prebattle` | 21 | 38–40 boxes before the player's first move, with the intro |
| battle lines | 1–4 each | lengths fine; the shapes are templates |
| `ch01_victory` | 20 | the seal is worked out again after Harl already said it |
| `ch01_tbc` | 6 | the tightest scene |

The patterns, most important first:

1. **The box is a report.** Its job is to get a fact to the player, so what
   the speaker wants from the listener never enters it: Tamsin's news as a
   past-tense sequence, two boxes listing what is on the paper, Aske's
   headcount, Tamsin confirming the number, the narration reading the
   orders out again.
2. **One rhythm for every voice.** Measured over the script, the median
   sentence is 4 words for Hollis, Maud, Rue, Aske and Harl and 5 for
   Tamsin; 51–63% of each speaker's sentences are 4 words or fewer. Maud's
   sheet asks for long polite sentences that interrupt themselves; the
   script gives her none. What tells the speakers apart is the form of
   address (noble, heir, kid, Keeper). The model's house style came out in
   a "serious" costume: clipped fragments for everyone.
3. **Lines that say their own subtext.** Hollis announces that he has spent
   the morning not handing over the letter; Tamsin's grief for Harl is a
   sentence of self-description; the death lines state regrets.
4. **Narration does the characters' work.** 8 of the intro's 19 boxes are
   narration. It points at silences ("has not said a word"), explains what
   taking the sword means, and speaks for the dead Hollis.
5. **Everyone cooperates.** Speakers take turns, one or two boxes each,
   mostly to the lead. Questions get full answers. The cast's built-in
   frictions (Aske and Tamsin fought on opposite sides; Hollis is cold to
   Maud) never meet on screen. The two refusals in the chapter (Harl's
   "Ask me when I'm dead", Rue turning the question back on the lead) are
   the two moments the critic found alive.
6. **Every box ends on a closer**, and the death and retreat lines follow
   two templates (the wound, then an order to finish; shock, then a
   bequest or a regret).
7. **The same facts again and again.** The seal is worked out four or five
   times; "four years" is said eight times; money and debt words about 35
   times, by three different speakers.
8. **"As you know" exposition.** "I'm sworn to fire, Keeper, and you know
   it"; the law of seals laid out step by step just after the boss said it.
9. **Lines that serve the game's data.** Aske's count exists to match the
   map. Tamsin's "it has to be all of them" was the reason for the old
   Rout objective and now contradicts Defeat Harl (0845).
10. **Scenes start early and end late**, and lean on stock lines any
    fantasy game could have.

What works, in the critic's reading (beats, not lines to keep: 0737
starts on a blank page): behavior instead of statement (Hollis folds the
letter back into his coat instead of burning it, and the lead later does
the same); Aske's entrance as one concrete image; the miller's wife who
overpays for rabbits, which shows Aske likes the hamlet without saying
so; Tamsin and Harl turning one shared idiom against each other; the
refusals; the grief objects in the Classic branches (a horse left
saddled, silver going north); and `ch01_tbc`, six boxes ending on a lie.

## What went wrong in 0736's first samples

The first before/after samples (three made-up scenes: a scout and a
sergeant arguing before an attack on a bridge-house, a priest asking for
a dead deserter's name, the deserter captain's battle lines) were written
to round 1's rules and judged best by two blind judges. Nick called the
people robots. A critic who agreed with him, and five researchers who
each held the samples against their sources, found the same things:

- **Every line was a move in a debate.** Claim, evidence, rebuttal: even a
  carter shot dead in front of a sixteen-year-old was filed as evidence
  for her plan. People who are scared and about to fight talk past each
  other, repeat themselves, pull rank, go quiet.
- **Nobody was scared, unsure, wrong or kind out loud.** Everyone had full
  information and a plan. A cast with no visible weakness gives the
  player nothing to protect, and protecting people is what permadeath
  runs on.
- **A sixteen-year-old talked like a veteran.** Understatement, a composed
  simile, a cool cross-examination: the same register as the sergeant.
- **The quips came back as dry wit.** "His mother had hopes", "Two Ts",
  "They were the crown's": crafted buttons that make the player admire the
  writer instead of feeling for the person.
- **Grief became a prop to decode, and every feeling bookkeeping.** A
  matched pair of tally sticks closed the grief scene; the whole set spoke
  in pennies, notches and months owed.
- **Nobody named a feeling, so nobody seemed to have one.** Total
  suppression is flat affect, which is how fiction writes robots. Subtext
  only reads against a cast where some people say it straight; Path of
  Radiance, our tone reference, lets Mist beg Ike not to leave her.
- **Nobody looked after anybody.** Every box was on task: nobody asked if
  the girl who lay in the mud all night was cold or had eaten.
- **The reply choices dispensed facts** instead of moving anyone.
- **The examples in the skill modeled the robot** and praised its bans,
  and examples are what steer a model hardest.

Which rules caused it, and how each changed in the skill: "every box does
something" became "every scene has people being people"; "facts are
ammunition" became "facts are said once, by someone who cares"; "nobody
says the feeling" became "small feelings out loud, the big one once";
"under threat, the same voice, shorter" became "each voice does its own
thing when scared"; the voice card gained age, temperature, openness, a
care signature and a fault; the cold read now asks who the reader would
miss; the examples were replaced.

## Warmth: what makes players care

Round 2's findings. Each line names where it shows.

- **An ordinary evening before the threat.** The deaths that land are set
  up by a scene with only small things at stake. *The Last of Us* spends
  its warm minutes on Sarah waiting up to give Joel a watch she saved for;
  Expedition 33 gives Gustave a quiet hopeful moment with Maelle just
  before Renoir strikes. The warmth comes before the threat, never in it.
- **Care lands on the body.** In every war story studied, love sounds like
  looking after feet, food, cold and wounds: Sam cooking rabbits for
  Frodo; Winters ordering Doc Roe to go eat something hot in Bastogne
  (*Band of Brothers*); after Greil dies, Oscar cooks and Ike orders
  Titania to bed (Path of Radiance Ch8). Studies of "invisible support"
  (Bolger and others) find practical help the person barely notices works
  best. It isn't funny, so it fits a battle played straight.
- **Answer the small bid.** Gottman found couples who stayed together
  answered each other's small bids for attention about 86% of the time,
  against 33% for those who divorced. Found-family games are built of
  these: Red Dead Redemption 2's camp, Path of Radiance's base talks,
  Hades between runs.
- **Small feelings out loud, the big one once.** StoryScope (Russell and
  others, 2026, 61,608 stories) found people name an emotion outright in
  29% of stories and models in 8%; models show it through the body 81% of
  the time against 38%. The lines players remember are plain admissions
  from people who rarely make them: Arthur telling Sister Calderon he is
  afraid (RDR2), Mist begging Ike not to leave her. Rationed to once per
  relationship per act, at a threshold, and followed by something
  practical, it reads as human, not as syrup.
- **Somebody is bad at something.** Bopp and others (CHI PLAY 2019)
  separate admiring a character's competence from worrying about their
  well-being, and a death needs the worry. Mordin's patter song, Arthur and
  Lenny's drunk night in Valentine a chapter before Lenny dies.
- **Teasing between two people, about a habit, with a repair.** Keltner
  (1998) found people who care about the person they tease do it with less
  bite and more repair. A quip is the writer performing for the audience;
  a tease only works between these two people. Never in battle.
- **Small talk while the big thing breaks.** Chekhov's Lopakhin and Varya
  talk about a lost item and the frost instead of the proposal; in
  *Manchester by the Sea* a boy breaks down over frozen meat because the
  ground is too hard to bury his father. Harvey Sacks: under strain people
  "do being ordinary".
- **Reach for the feeling and miss.** Lonergan's sidewalk scene: Randi
  keeps breaking off and restarting her apology while Lee tries to stop
  her. The clumsy attempt is where the warmth is.
- **Disagree like people who like each other.** Conversation research
  (Pomerantz 1984) finds people agree fast and disagree late, with padding;
  quick clean rebuttals are how strangers talk. When someone runs
  themselves down, the friendly answer is to contradict them, fast.
- **The one in pain laughs; the friend doesn't.** Jefferson (1984): a
  teller laughs to make trouble smaller, and the listener stays serious to
  show they're taking it in.
- **Shared history as an old argument worn smooth.** Aubrey and Maturin,
  Sharpe and Harper: the fortieth round of the same small disagreement,
  never settled, which nobody is left to finish when one of them dies.
  Final Fantasy Tactics' grass whistles: when the talk stalls, the old
  ritual.
- **Plant something small; let the death cut it off.** Matthew invites
  Leila to meet his parents and walks off before she answers; two chapters
  later she is dead and his grief lands on that (The Blazing Blade). Greil
  starts on Ike's mother during a night walk, sends him back, and is dying
  minutes later. Arthur admits he'd like to be buried facing west.
- **Last words are about the living.** Greil spends his last breath on
  chores (look after Mist, take over the company, don't chase the Black
  Knight); Ninian is glad Eliwood wasn't hurt; Jeralt notices Byleth's
  first tears.
- **The one hurting most says least.** Matthew's first answer to Leila's
  death is a box of dots, then a deflection, then a small broken laugh,
  then a practical request.
- **The dead are remembered with a small, unflattering, true story.**
  Tim O'Brien remembers Curt Lemon fainting at the army dentist's, not a
  list of virtues. *M\*A\*S\*H*'s Abyssinia, Henry spends the episode on
  goodbyes and ordinary time; the news lands in the OR and they keep
  operating.
- **The cast notices the silent lead.** Greil's night walk asks Ike how
  he's settling in; Thane, dying, prays for Shepard; Valve's writers say
  players know Gordon by how others react to him.

What to avoid, from the same sources: syrup (speeches about family, the
cast praising the lead, the big feeling said often), therapy words, a
symbol carrying grief with no setup, and Becky Chambers-style cooperative
fluency where everyone names their needs, which is the register Nick
called robots.

## Why AI characters read as robots

- **The model's default character is a consultant.** Anthropic's own
  research (the Persona Selection Model, Feb 2026; Lu and others, *The
  Assistant Axis*, Jan 2026) describes a chat model as one character, the
  Assistant, kept near the consultant, coach and evaluator end of the
  space of personas. Its trained traits (curiosity, balanced judgment,
  patience, warmth with boundaries) are the traits of the rejected cast.
  Stanford's Generative Agents (2023) talked too formally and agreed too
  easily.
- **Alignment trims pettiness, fear and self-interest.** Yi and others
  (*Too Good to be Bad*, 2026) found role-play fidelity falls as a
  character's morality falls; Tian and others (EMNLP 2024) found model
  stories uniformly positive and low in tension. StoryScope's section on
  Claude finds the least escalation and the most uniform voice of any
  source.
- **Free improv between model agents makes it worse.** Two Claude
  instances left to talk drift into mutual gratitude and calm (Bowman and
  Fish, *Asterisk*). What has helped in research: giving each actor
  private motives and inner thoughts (CoSER, ICML 2025). So the skill's
  optional improv gives each character agent a private want and a hidden
  thing and never lets them chat freely.
- **Asking for several versions helps; "make it messier" is folklore.**
  Verbalized sampling (Zhang and others, 2025) made creative writing 1.6–2.1
  times more varied: hence "draft five, keep a plain, less likely one".
  Temperature does little (and our pipeline can't set it).
- **Changing models changes the tics, not the problem.** Models resemble
  each other more than they resemble people (StoryScope; Chakrabarty,
  Laban, Wu 2025; Reinhart and others 2025).

## Which model

Current routing is the `creative` capability in `tickets/README.md`
(optional examples: Fable / Astra). The dated findings below record the
models and drafts actually tested; they do not require a provider switch
or compare a model that was not tested.

Nick asked whether a better model for writing exists. The finding: **no
model is clearly better than Claude for human, emotional dialogue**, and
within Claude the differences are small. Benchmarks as read on 2026-10-23
and 24:

| Model | EQ-Bench Creative Writing v3 (Elo) | EQ-Bench Longform | Lech Mazur short stories | LMArena creative writing (human vote) |
| ----- | ---------------------------------- | ----------------- | ------------------------ | ------------------------------------- |
| GPT-6 Astra | 2173 (1st) | 82.8 | 3.443 | 39th |
| Claude Fable 5.1 | 2162 (2nd) | 85.3 | 3.756 (1st) | 8th |
| Claude Opus 5 | 2133 (3rd) | 86.3 (1st) | 3.730 at xhigh effort | 14th |
| Claude Opus 5.5 | 2050 (7th) | 84.6 | 3.715 | 1st |
| Grok 4.7 | 2007 (8th), best judged dialogue | 83.0 | 25th | not in top 60 |
| Gemini 4 Argon | not listed | not listed | 19th | 2nd |

- The judged boards are scored by a Claude model and bunched at the top;
  human votes reward style on short prompts; **nothing measures whether a
  player would miss a character.**
- Opus 5.5 (what the story tickets run on today) is a little behind Fable
  5.1 and Opus 5 on the judge's show-don't-tell, believable-characters and
  emotional-depth scores, and has more stock phrases. The gaps are 0.2–0.5
  points out of 20.
- Fiction-specialized options lose: Sudowrite Muse has no API (a novelist
  reports its dialogue weak); NovelAI's model is a fine-tune of an older
  base with a short context; community fiction fine-tunes trail by
  700–1000 Elo and the best is licensed non-commercial.
- Every major vendor gives the customer ownership of outputs; an outside
  model would need Nick to open an account and a script to send briefs,
  and its drafts would still pass our checks.
- Expert readers in Chakrabarty and others (2026) marked down prompted
  output from every vendor alike. That fits round 2's finding: the robots
  came from our brief and examples more than from the model.

So round 2 tested the two cheap options blind: Opus 5.5 against Fable 5.1,
each with the new skill alone and with the character method. The result is
under "Round 2's samples" below.

## Round 2's samples

Four writers wrote the same three-scene arc from the same cards (a quiet
night before an attack, the morning of it, the sergeant's death and the
night after): Opus 5.5 and Fable 5.1, each with the revised skill alone
and with the character method (private wants and fears, a first-person
monologue, five drafts of the key boxes). Two judges read them blind: one
told Nick's tastes and rejections, one a plain player given no rules or
intentions at all. Scores out of 10:

| Writer | Nick-like judge: miss Osric / like Wenna / human | Cold reader: miss Osric / like Wenna / human | Rank (Nick-like, cold) |
| ------ | ------------------------------------------------ | --------------------------------------------- | ---------------------- |
| Fable 5.1, skill alone | 9 / 9 / 9 | 9 / 9 / 8 | 1st, 1st |
| Opus 5.5, skill alone | 8 / 8 / 8 | 6 / 7 / 6 | 2nd, 4th |
| Fable 5.1 with the method | 8 / 8 / 7 | 8 / 8 / 7 | 3rd, 2nd |
| Opus 5.5 with the method | 6 / 7 / 7 | (not scored) | 4th, 3rd |

- **Both judges picked the same draft** and the same best line ("Who's
  going to do his sister's door?"). It is the skill's example, with the
  judges' small fixes (a few unclear words, one British word, a
  continuity slip, two punchline tails).
- **The model mattered more than the method.** Fable beat Opus with and
  without the method; the method gave no clear gain (it helped Opus with
  one judge and hurt it with the other). So the skill says to write
  scripts on Fable 5.1 and keeps the method optional.
- What the judges still flagged in the weaker drafts: grief turned into a
  comedy bit (a goose story told the night of the death), a "sorry" tic so
  strong two voices blurred, a tidy callback as a last line, sitcom beats
  ("which I don't, obviously, I'm sixteen").

## Round 3: punctuation, and nudges

Nick on the round 2 winner: "it's better, still a bit off. This is in the
right direction though. One thing I don't like is the overuse of a period
to end a sentence. A period is a fact, an ellipses or an emdash (as AI as
it looks) at least implies some trailing thought. And what about
exclamation marks?" Counted: 86% of its sentences ended on a period, with
no exclamation mark at all, even from the frightened sixteen-year-old.
(Fire Emblem sets temperature this way: Path of Radiance's Mist uses about
12 exclamation marks per 100 words, her brother Ike under 2.)

Three Fable 5.1 drafts followed (a delivery pass on the winner and two
fresh drafts), judged blind by the same two judges, who split: the
Nick-like judge preferred a livelier fresh draft, the cold reader the
quieter pass. Both, fixed, went to Nick. His answer:

> "now this Wenna character is too annoying, exclamations in every single
> line they give. Idk, there needs to be a balance. You are taking my
> instructions at face value, when I need nuance. I ask for more
> exclamations and now every line from this character has them, now she's
> annoying. There just needs to be some more expression is what I meant
> the first time, but now she's like Tiny Tina and that character didn't
> age well."

The lesson is about notes, not only punctuation: **every rejection in
this file came from obeying the last note literally and swinging to its
opposite extreme**, losing what the version before got right. The skill
now says so at the top of its Craft section. The final sample keeps the
round 2 scenes and moves one notch: Wenna exclaims in 2 of her 15 boxes
(the laugh at the privy door, pleading to wait for dark), Osric once
(dying), and the rest of the expression is questions, trailing thoughts
and cut-offs; about 70% of sentences still end on a period. That calibration
is in the skill's "Punctuation is the voice acting".

## Why AI dialogue reads bland

Round 1's findings: what the studies and the writers who have edited model
fiction say, and how each failure showed in Chapter 1:

| Failure | What it is | Source |
| ------- | ---------- | ------ |
| Report voice | Information in tidy, noun-heavy sentences; nobody wants anything | Reinhart et al., PNAS 2025 (instruction-tuned models write in an informational register in every genre); Mirowski et al., Dramatron, CHI 2023 (professionals: too much information, said too explicitly) |
| One voice in every mouth | Every speaker in the model's house style | Reinhart et al.; Chakrabarty et al., C&C 2024 (an MFA writer never got varied sentence structures) |
| Feelings named | Characters state emotions and intentions | Chakrabarty et al., *Art or Artifice?*, CHI 2024 (almost no subtext); Dramatron |
| Subtext translated | One line implies, the next explains | Chakrabarty, Laban, Wu, CHI 2025: redundant exposition in about 18% of professional edits |
| Everyone answers | Direct, polite, complete answers; nobody stops talking | Ivey et al., *Real or Robotic?*, 2024–26 |
| Agreement on cue | Conflicts resolve within a box or two; upbeat arcs | Tian et al., EMNLP 2024 (setbacks resolved quickly; downward arcs nearly vanish) |
| Villains without teeth | Bosses soften or only shout | Yi et al., *Too Good to be Bad*, 2025 |
| The closing moral | A neat summing-up last line | Chakrabarty et al., C&C 2024 (moralizing, predictable endings); Wikipedia, *Signs of AI writing* |
| Speechwriter shapes | "Not X, but Y"; lists of three | Paech et al., *Antislop*, ICLR 2026 (the not-X-but-Y frame at about 6× the human rate in some models); Wikipedia |
| Generic detail | Categories instead of things | Chakrabarty, Laban, Wu (lack of specificity); Xu et al., *Echoes in AI*, PNAS 2025 |
| The same scene shape | Worry, reassurance, resolve, aphorism, repeated | Xu et al.; Wenger and Kenett 2025 (different models' outputs resemble each other more than humans' do) |
| Scenes that won't end | Thanks, agreement and recaps after the turn | Ivey et al.; Tian et al.; EQ-Bench's *Meandering* |
| Polishing a flat skeleton | Line edits that keep the scene's order and beats | Tian et al.: suspense rated best rose from about 8% to about 48% only when turning points were planned first |
| Interviews around the silent lead | Every NPC addresses the lead and waits | (an observation; no study found) |

What this means for the process, which matters more than any single rule:

- **A longer rule list doesn't fix it.** One project
  (`zenstory-ai/oh-story-claudecode`) wrote down a test in advance and
  found that countable style rules in the writer's prompt, or a
  detect-then-patch loop, did not cut the AI tells enough. The *Antislop*
  paper finds that telling a model to avoid words can prime them. EQ-Bench's
  older judge code notes that showing its criteria to the model being
  tested seemed to make its writing worse. So the skill gives the writer
  positive craft and examples, and keeps the fault list for the critique
  pass.
- **Examples steer hardest, and leak.** Anthropic's prompting guide says
  examples are the most reliable way to steer tone; roleplay guides warn
  that sample lines leak into output word for word. A report-shaped sample
  line produces a character who reports.
- **A model grading its own draft is biased.** EQ-Bench doesn't control for
  it. The skill's critique pass uses a fresh agent that reads only the
  boxes, and Nick's verdict stays final.
- **Plain-prose rules are the wrong cure for dialogue.** Anti-slop skills
  (humanizer, stop-slop, Wikipedia's list) are built for expository prose;
  their pressure toward plain, literal statement is the pressure that
  produced the scout's headcount. They are for narration, menus and tips.

## Writing resources made for AI writers

Looked at on 2026-10-23. Licences as stated in each repository's LICENSE
file, metadata or README. **Nothing was copied**: everything the skill
took is an idea, written in our own words, and the table says where it
came from. Only MIT, Apache-2.0, BSD, ISC, Zlib, CC0 or CC-BY-4.0 text
could have been copied under ADR-0013.

| Resource | Licence | What it teaches | Worth for us |
| -------- | ------- | --------------- | ------------ |
| [anthropics/skills](https://github.com/anthropics/skills) (Anthropic) | Mixed: frontend-design Apache-2.0; doc-coauthoring none stated | No skill for fiction or dialogue (19 skills); the official plugin marketplaces have none either | Low as content. Two methods borrowed: frontend-design opens its brief with "the client has rejected templated work" and names the defaults to avoid; doc-coauthoring's reader test (a fresh model with no context) became the cold read |
| [oh-story-claudecode](https://github.com/zenstory-ai/oh-story-claudecode) (zenstory-ai), `dialogue-mastery.md` | MIT | A Chinese web-novel pipeline: dialogue patterns by scene job, bans, self-checks, and a write-up of failed anti-AI-flavor experiments | High. Names our failure (lines take turns broadcasting facts instead of answering the last speaker's feeling); turn length shows who holds power; the experiment above |
| [jwynia/agent-skills](https://github.com/jwynia/agent-skills), fiction skills | MIT in each file's frontmatter; no LICENSE file | Diagnostic states for dialogue (identical voices, wooden, exposition dump, on the nose), scene-and-sequel pacing | High as a checklist: hidden agendas, the "tennis match" of evenly sized turns, scenes ending on "yes, but" or "no, and" |
| [haowjy/creative-writing-skills](https://github.com/haowjy/creative-writing-skills) | Apache-2.0 | A multi-agent novel pipeline; AI-fiction antipatterns by strength of evidence; character and reader simulation | High for diagnosis ("reported, not lived"; everyone equally articulate) and process (interview a character before writing; a cold reader says where they'd skip) |
| [bybren-llc/story-systems-template](https://github.com/bybren-llc/story-systems-template), dialogue-craft | MIT | Screenwriting: four kinds of subtext, exposition through conflict, power shifts | Medium-high: the same fact said in three voices; who holds power at the start and the end |
| [msitarzewski/agency-agents](https://github.com/msitarzewski/agency-agents), narrative designer | MIT | A game narrative designer persona: voice pillars, choice rules | Medium: reply choices must differ in kind and each get a distinct reaction |
| [vibeforge1111/vibeship-spawner-skills](https://github.com/vibeforge1111/vibeship-spawner-skills), narrative sharp edges | Apache-2.0 per README; no LICENSE file | A catalogue of game-narrative failures, the silent protagonist among them | Medium: the world must notice the silent lead |
| [blader/humanizer](https://github.com/blader/humanizer) | MIT (built on Wikipedia's CC BY-SA page) | 26 patterns of AI prose | Low for dialogue, useful for narration: rewrite around the point instead of patching phrases |
| [hardikpandya/stop-slop](https://github.com/hardikpandya/stop-slop) | MIT | Eight rules toward clipped, specific, factual prose | Low and risky: it rewards the report voice |
| [Wikipedia: Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) | CC BY-SA 4.0 (ideas only) | Tells of AI text in encyclopedic writing | Medium: "regression to the mean" (the likely word replaces the specific one) is the clearest account of why drafts read bland |
| [EQ-Bench Creative Writing v3](https://github.com/EQ-bench/creative-writing-bench) (Sam Paech) | None stated (ideas only) | How a leaderboard judges model fiction: 22 criteria, pairwise rounds in both orders at equal length | High for the critique pass: judge two whole drafts against each other; analysis before scores |
| [EQ-Bench legacy judge](https://github.com/EQ-bench/EQ-Bench/tree/main_v2_4) | MIT | The earlier criteria, with a longer fault list | High: named faults such as tit-for-tat dialogue, characters too good, profundity over-reach |
| [EQ-Bench Longform](https://github.com/EQ-bench/longform-writing-bench) | MIT per README | A writing pipeline with a step that critiques the plan for model habits before any line exists | High: the scene card's "obvious staging, then a different one" |
| [antislop-sampler](https://github.com/sam-paech/antislop-sampler), [slop-forensics](https://github.com/sam-paech/slop-forensics), [*Antislop* paper](https://arxiv.org/abs/2510.15061) | Apache-2.0; MIT; CC BY 4.0 | Lists of over-used phrases and names; profiling text against a human baseline | Medium: check narration after drafting, never ban words in the prompt; profiling each speaker's lines could measure "one rhythm" |
| [Sudowrite docs](https://docs.sudowrite.com/) and blog | None stated (ideas only) | Story bible, style lines, one scene as one unit | Medium: length follows the number of scene jobs; its advice to splice the best lines of several drafts is what we avoid |
| [Novelcrafter](https://www.novelcrafter.com/help/faq/ai-and-prompting/ai-isms) | All rights reserved (ideas only) | Style entries; AI-isms highlighted, not prompted against | Medium: too many rules make prose stilted, so the skill's checklist has seven questions |
| [Anthropic prompting docs](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices) | None stated (ideas only) | Examples steer tone most; match the prompt's style to the output | High: why the sheets' sample lines mattered; write a scene card in the scene's mood, not as facts |
| [OpenAI Model Spec](https://github.com/openai/model_spec) | CC0-1.0 | "Be creative": go past the obvious; labelled BAD/GOOD pairs | Medium: the before/after format |
| [SillyTavern docs](https://docs.sillytavern.app/usage/core-concepts/characterdesign/), [Ali:Chat guide](https://rentry.co/alichat) | All rights reserved / AGPL repo; none stated | Roleplay character cards | Medium: the first exchange sets length and tone; samples leak |
| [Gwern, GPT-3 Creative Fiction](https://gwern.net/gpt-3) | CC0 | The context decides which writer you get | Low-medium: write the first box with the most care |

**Verdict:** no existing resource fixes bland game dialogue by itself, and
none covers our format (a 200-character box, two portraits, no stage
directions, a silent lead with reply choices, battle lines, a box budget).
The skill's Craft section is our own, built from the ideas above and the
games and books below.

## Craft teaching

| Source | The idea we took |
| ------ | ---------------- |
| David Mamet, memo to the writers of *The Unit* (2005, [reprint](https://www.slashfilm.com/a-letter-from-david-mamet-to-the-writers-of-the-unit/)) | Every scene: who wants what, what happens if they don't get it, why now. No "as you know". Let the camera explain; for us the battle map is the camera |
| Robert McKee, *Dialogue* (2016), via [Kirkus](https://www.kirkusreviews.com/book-reviews/robert-mckee/dialogue-art/), [Matt Bell](https://mattbell.substack.com/p/55-using-cue-words-to-structure-better) and a [workshop recap](https://www.nowwwriters.ca/noww-blog/dialogue-shouldnt-say-it-out-loud) | Exposition as ammunition; what is said, felt but unsaid, and can't be said; end a line on the word that completes it |
| Sol Stein, *Stein on Writing*, via [Charles Deemer](https://www.ibiblio.org/cdeemer/cfs0901.htm) | The reply answers the meaning, not the words; deflection: change the subject, answer with a question, silence, counter-attack |
| Ursula K. Le Guin, *Steering the Craft* and "From Elfland to Poughkeepsie" (1973) | Telling It Slant: a story in dialogue alone, A and B, no stage directions (our format exactly); Short and Long (sentence length as a tool); the noun-swap test |
| Elmore Leonard, 10 Rules (NYT 2001, [summary](https://fs.blog/elmore-leonard-10-rules-of-writing/)) | Dialect from word choice and rhythm, not spelling; the line carries the attitude |
| Hannah Nicklin, [dialogue workshop](https://gutefabrik.com/craft-writing-better-dialogue-a-2-day-workshop/), and [Emily Short's review](https://emshort.blog/2022/07/05/writing-for-games-theory-practice-hannah-nicklin/) of *Writing for Games* | A voice sheet per speaker; the names-removed test; weak comedy gives everyone the same joking voice |
| Valve's writers, [GDC Online 2011](https://gamedeveloper.com/design/gdc-online-valve-writers-candid-thoughts-on-creative-process) | A silent lead is a real limit: banter planned for two fails when one can't talk; players know Gordon by how others react to him |
| Greg Kasavin on [Bastion](https://www.gameinformer.com/b/features/archive/2011/09/09/bastion-afterwords.aspx) and [Hades](https://gdcvault.com/play/1026975/Breathing-Life-into-Greek-Myth) | Short reactive lines win on specificity; cadence matters nearly as much as words |
| Neil Druckmann and Troy Baker on Joel's death ([via DualShockers](https://www.dualshockers.com/the-last-of-us-part-ii-neil-druckmann-ps4/)) | The last line about his daughter was cut; the scene is stronger without it. Try the version where nothing is said |
| Mark Slabinski, [8 principles of game dialogue](https://www.gamedeveloper.com/game-platforms/8-key-principles-of-writing-effective-game-dialogue) | A 62-word quest pitch cut to about 20 with nothing lost; one bark carries one piece of state |

## What the studied games and books do

Each entry: the techniques, a scene where each shows, and what to avoid.
Scene descriptions are in our own words; the sources are listed after
each.

### Fire Emblem: The Blazing Blade and Path of Radiance

Path of Radiance is our tone reference.

- **A briefing is doubt and a decision, with the facts moved out.** In
  *A Brief Diversion* (PoR Ch6), Soren can't count the pursuers and argues
  for splitting the company; Oscar objects; Greil demands a best guess,
  decides, and closes on the company as family. Titania's Hard-mode
  briefing in Ch1 is one sentence; the map shows the rest. FE7 puts
  geography and politics on a short narration card so no one recites them.
- **Write to the silent lead, not around him.** FE7's tactician never
  speaks. In the good scenes the other character reacts to his unheard
  answer (Matthew wheedling after being refused a chest in *Blood of
  Pride*); in the weak ones they ask his opinion and agree in the same
  breath. Ike is terse: 54% of his lines are 40 characters or fewer, with
  1.6 exclamation marks per 100 words against his sister Mist's 12.3.
- **Grief in the wrong words.** If Oscar falls in PoR Ch1, his brother Boyd
  says he's glad nobody will nag him now, then breaks on the name. In
  FE7's *The Dread Isle*, Matthew calls Leila's death her own mistake, then
  admits he meant to ask her to quit after this job.
- **Boss talks keyed to shared history.** Each boss has one line for anyone
  and one on defeat; longer talks go only to those who know him (General
  Eagler facing three of his former knights, FE7). Death quotes are short
  (PoR median 55 characters) and about 40% name a person or a cause.
- **Plant a phrase, pay it off.** The taunt Greil uses sparring with Ike in
  the Prologue is the one he throws at the Black Knight in the duel that
  kills him.
- **Avoid:** characters summing up the plot to each other (Pent on the
  Shrine of Seals); asking the silent lead, then agreeing at once;
  interchangeable "ready" lines; late FE7 chapters with about 120 turns
  before the battle.

Sources: Fire Emblem Wiki scripts ([FE7](https://fireemblemwiki.org/wiki/Category:Scripts_of_Fire_Emblem:_The_Blazing_Blade),
[PoR Ch1](https://fireemblemwiki.org/wiki/The_Battle_Begins/Script)), Serenes
Forest ([PoR Ch6](https://serenesforest.net/path-of-radiance/scripts/game-script/chapter-6-a-brief-diversion/),
[death quotes](https://serenesforest.net/path-of-radiance/scripts/game-script/miscellaneous/death-quotes/)),
the [PoR localization interview](https://nintendoworldreport.com/interview/2266/fire-emblem-path-of-radiance-interview)
(the team counted button presses so long talks wouldn't wear players
out), Audrey Cheng's [GDC 2018 narrative review](https://media.gdcvault.com/gdc2018/GameNarrativeReview/AudreyCheng_GDCGame%20Narrative%20Review.pdf).

### Fire Emblem: Three Houses

- **One fact, three voices.** The prologue's three house leaders bicker
  over whether Claude fled the bandits or was playing decoy; the same
  event, bent through three worldviews, and the leaders argue with each
  other, not with the professor.
- **The cast narrates the silent lead.** Byleth speaks almost only in
  ellipses; others report the lead's face. Jeralt's last lines point out
  that these are Byleth's first tears.
- **Deny the feeling, prove it with an act.** Hubert and Ferdinand's top
  support ends with each having bought the other's favorite drink; Felix
  claims he only spoke up because Dimitri looked pathetic.
- **A voice is a line length, a word stock and one refusal, and the arc
  bends it.** In supports Felix's median line is 45–64 characters and
  Dimitri's 62–92. After the timeskip Dimitri goes from long, hedged
  sentences to short commands, and never says Edelgard's name.
- **Humanize the enemy before the fight.** Randolph gets a scene with his
  sister before Dimitri executes him; three chapters later she kills
  Rodrigue.
- **Avoid:** war councils of 300–418-character tactical speeches; supports
  with Byleth that turn into interviews (the partner's lines run 1.5–2×
  longer); generic cheering roll calls before the finale; the same
  gameplay hint said by two characters.

Sources: Fire Emblem Wiki scripts ([Ch9](https://fireemblemwiki.org/wiki/The_Cause_of_Sorrow/Script_(Blue_Lions)),
[prologue](https://fireemblemwiki.org/wiki/An_Inevitable_Encounter/Script),
[Azure Moon Ch17](https://fireemblemwiki.org/wiki/Blood_of_the_Eagle_and_Lion/Script_(Azure_Moon))),
[Hubert's supports](https://fireemblemwiki.org/wiki/Hubert/Supports),
essays in [Gayming Mag](https://gaymingmag.com/2019/11/fire-emblem-three-houses-non-byleth-relationships-feel-more-significant/)
and [Kotaku](https://kotaku.com/its-kind-of-creepy-how-obsessed-professor-byleth-is-wit-1838183434).

### Tactics Ogre: Let Us Cling Together

- **Exposition as a quarrel.** The PSP opening is a wordless cold open of
  a burning town, then about 12 lines of three friends arguing over an
  ambush. The politics arrive as their reasons.
- **Both the victims and the order get a real case.** Before the Balmamusa
  choice, the refugees argue coherently for staying out of the war; the
  Duke's reasoning is shown in his own cold words. The choice is an answer
  to a named person's order.
- **One event, several moral engines.** After Balmamusa each companion
  applies their own fixed rule to it in a box or two; on the other route
  Vyce takes the other side, showing that the rule matters more than the
  side.
- **The courteous enemy.** Lanselot Tartaros hosts the young men whose
  town his knights burned, gruff and polite, and apologizes uselessly for
  an old wrong.
- **Register marks rank, without thee and thou.** Alexander O. Smith's team
  aimed for a laconic style, called thee and thou "disruptive almost all
  the time", and read each character's lines on their own to catch a voice
  that didn't fit.
- **Avoid:** a proper-noun avalanche before the player cares; reveal
  stacked on reveal; archaic register as a cure for bland (old syntax
  still reads like a report if nobody wants anything).

Sources: [Kajiya Productions interview](https://www.gamedeveloper.com/game-platforms/in-depth-kajiya-productions-on-the-art-of-localizing-i-tactics-ogre-i-),
[Hardcore Gaming 101](https://www.hardcoregaming101.net/tactics-ogre/), the
[LP Archive playthrough](https://lparchive.org/Tactics-Ogre-Let-Us-Cling-Together/),
[RPGFan's Reborn review](https://rpgfan.com/review/tactics-ogre-reborn).
Scene details come from a let's-play, not a full transcript.

### Final Fantasy Tactics (The War of the Lions)

- **Let the enemy give the briefing.** Many Chapter 1 battles open on the
  enemy talking among themselves: at the Sand Rat's Sietch the Corpse
  Brigade men agree to desert and hope the ransom pays them out. The odds
  and a reason to pity them come in the same few lines.
- **A short fuse before the fight, the argument inside it.** Most Chapter 1
  battles have 2–9 spoken lines before play starts. Milleuda's and
  Argath's argument about class comes on turn triggers during the fight;
  Wiegraf speaks on his tenth turn.
- **Class through grammar, not thee and thou.** The War of the Lions script
  uses thee and thou zero times; nobles speak in inverted formal
  sentences, Delita plainly and in local words. When Delita explains a
  desert word, Argath's three bare "...?" mark him as the outsider noble.
- **Words that change hands.** Argath calls commoners chattel, Milleuda
  throws the word back, Delita uses it to defend her. A grass-blade
  whistle shared in Chapter 1 returns in Chapter 4.
- **A plot fact delivered as a blow.** Retreating at low HP, Wiegraf tells
  Ramza that his own brother arranged the kidnapping.
- **Avoid:** florid words that stop the eye (nescient, an archaic word
  for corpse); villain
  voices that all sound alike; the theme announced as a debate.

Sources: the [Final Fantasy Wiki's War of the Lions script](https://finalfantasy.fandom.com/wiki/Final_Fantasy_Tactics:_The_War_of_the_Lions_script)
(1,968 lines measured), [RPGFan on its voice](https://classic.rpgfan.com/features/Final-Fantasy-30th-Anniversary/The-Lion-War-Finding-the-Voice-of-Ivalice/index.html),
[Destructoid's review](https://www.destructoid.com/destructoid-review-final-fantasy-tactics-war-of-the-lions/).
(The War of the Lions was translated by Joseph Reeder and Tom Slattery.)

### Triangle Strategy

- **Each character is the argument that moves them.** Before the Chapter
  VII vote on handing Roland over, each companion answers to a different
  kind of reason (morals, facts and the house, loyalty, a workable plan),
  and each has an argument they won't accept.
- **A fact is ammunition, with a price.** Hidden oil in Castle Wolffort can
  win Benedict over to fighting, at the cost of burning homes.
- **The outvoted stay angry.** Hughette shows it when the vote goes
  against her; on each route one of the three main companions walks
  away.
- **Each enemy argues a case a friend half-shares.** Aesfrost promotes for
  ability, not birth.
- **Avoid:** the volume (MobileSyrup reached one battle in about two and a
  half hours; Eurogamer: a battle is often followed by over an hour of
  cutscenes); one scene's information restated in two more (CGMagazine);
  heroes who are, in IGN's words, "excruciatingly, painfully, polite at
  all times"; one stiff old-fashioned register for everyone.

Sources: [Wikipedia's reception summary](https://en.wikipedia.org/wiki/Triangle_Strategy),
[Kotaku on the votes](https://kotaku.com/triangle-strategy-square-enix-scales-of-conviction-vote-1848657909),
[IGN's review](https://tech.yahoo.com/gaming/articles/triangle-strategy-review-202908829.html),
[CGMagazine](https://www.cgmagonline.com/review/game/triangle-strategy-switch-review),
[MobileSyrup](https://mobilesyrup.com/2022/03/12/triangle-strategy-nintendo-switch-review/).
No public English script exists, so this is from reviews and walkthroughs.

### Unicorn Overlord

- **Short scenes; the map and the library hold the facts.** Most battles
  have a short scene before and after. Its director's rule: the script
  may not claim an army the map doesn't show.
- **A rapport talk is about one thing**, and no talk runs longer than a
  minute or two (Nintendo Life).
- **A manner and its contradiction.** Each recruit is pitched as one pair:
  refined manners over a lethal blade; a quick temper devoted to her
  soldiers.
- **The lead connects; the partner drives.** Alain's best rapport talks are
  with partners who arrive wanting something from him; the weakest are
  with partners who have no tie to him.
- **Avoid:** short alone is not enough. Reviewers still called the main
  story dry and serious with a flat hero: short lines with no voice read
  like our reports.

Sources: [RPG Site](https://rpgsite.net/review/15687-unicorn-overlord-review),
[Nintendo Life](https://www.nintendolife.com/reviews/nintendo-switch/unicorn-overlord),
[Destructoid](https://www.destructoid.com/reviews/review-unicorn-overlord/),
[Denfaminico Gamer interview](https://news.denfaminicogamer.jp/interview/230924c/2).
No transcript was reachable.

### Octopath Traveler I and II

- **Terse lead, colorful room.** In Olberic's first chapter he answers in a
  few words, each a commitment; the villagers carry the dialect, the jokes
  and the subtext.
- **A habit that is a wound.** Osvald (II) counts guards and prisoners after
  five years in prison. The numbers read as obsession and dread, not as a
  report. (Our scout could have been this: the second count has to mean
  something.)
- **The boss reads the hero.** Gaston, an ex-mercenary turned brigand chief,
  owns his crimes after losing and tells Olberic what his eyes look like.
  Simeon spreads his attack across a whole chapter.
- **One small question, many answers.** The tavern banters ask something
  tiny and let each traveler answer from their own values.
- **Avoid:** H'aanit's dialect (invented for the English version; hard to
  read for native and non-native readers alike); 31 of 32 chapters with the
  same shape; a party that is never in the room (what II fixed with
  Crossed Paths); a refrain repeated past its life.

Sources: the [LP Archive transcription](https://lparchive.org/Octopath-Traveller/),
Longfei Gao's [GDC 2019 narrative review](https://media.gdcvault.com/gdc2019/GameNarrativeReview/Gao_Longfei-paper.pdf),
[Kotaku on its old English](https://kotaku.com/two-very-different-jrpg-approaches-to-old-fashioned-eng-1828132765),
[the II developers on the protagonists' interaction](https://nintendoeverything.com/octopath-traveler-ii-devs-on-having-the-protagonists-interact/).

### Novels: Abercrombie, Cook, Martin

- **Facts arrive inside a want.** *The Heroes* opens with Craw, scared and
  with aching knees, climbing a hill to offer the man holding it a way out.
- **A refrain that changes weight** each time it comes back (Logen's line
  about being realistic; Craw's tired refrain about the times).
- **Talk around the wound.** Abercrombie's Northmen say a dead man has gone
  back to the mud; Croaker, the Black Company's annalist, leaves out what
  he won't say about his brothers.
- **Jokes live in the wait and come from a feud** (One-Eye and Goblin's
  prank war fills the Company's dull stretches).
- **The ensemble reads the silent one.** The Company's mute wizard Silent:
  the others read his looks aloud, guess, and argue over them.
- Abercrombie's advice: "Ask yourself with every line of dialogue, would
  someone really say this." He credits Elmore Leonard and James Ellroy.
- **Avoid:** dialogue that is clever and nothing else (the A.V. Club on
  *The Heroes*); a tic repeated after it has lost its charge; grim without
  warmth.

Sources: [Abercrombie interview](https://authorinterviews.co.uk/?p=416),
[Abercrombie on his blog Q&A](https://josephmallozzi.com/2008/05/23/may-23-2008-author-joe-abercombie-drops-in/),
[Glen Cook interview](https://strangehorizons.com/wordpress/non-fiction/articles/interview-glen-cook/),
[Strange Horizons on *The Heroes*](https://strangehorizons.com/wordpress/non-fiction/reviews/the-heroes-by-joe-abercrombie/),
[Reactor on the Black Company](https://reactormag.com/glen-cooks-the-black-company-is-grimdark-but-never-hopeless/).

### Novels: Pratchett, Lynch, Le Guin, Hobb

- **The swap test.** Le Guin took two lords' exchange from a fantasy novel,
  changed about four nouns, and it read as a Washington thriller. Her
  verdict on that kind of prose: "It is not really simple, but flat."
- **Humor grows out of the bad situation and goes quiet at the worst.** In
  *Guards! Guards!* the Watch fuss over making their shot exactly a
  million to one: fear dressed as procedure, and the dragon stays real.
  *Night Watch*'s torture cellar gets no jokes; Pratchett compared its
  humor to M\*A\*S\*H.
- **The mentor teaches by what he won't do or say.** Ogion lets the rain
  soak them both rather than spend a spell; Burrich crops his own hair in
  mourning and never says what he feels for Fitz.
- **Exposition only as ammunition.** Lynch regrets that his history
  interludes became lectures and wishes they had been Chains telling
  stories.
- **Hold one register.** Le Guin faults writers who switch to slang for the
  jokes and to formal speech for the earnest parts. Granny Weatherwax is
  funny and frightening for the same reason: she never changes.
- **The one detail this speaker would notice.** Lynch faults his own action
  scenes for exact distances and too little panic: precision is not
  concreteness.

Sources: Le Guin, ["From Elfland to Poughkeepsie"](https://en.wikipedia.org/wiki/From_Elfland_to_Poughkeepsie),
[Steering the Craft](https://www.ursulakleguin.com/steering-the-craft),
[Pratchett in *Writing Magazine*](https://www.lspace.org/about-terry/interviews/writmag.html),
[Lynch on his own book](https://www.goodreads.com/author_blog_posts/2260191-locke-lamora-read-along-bonus-4-you-suck-lynch),
[a Hobb reread](https://elliottrwi.com/2019/06/14/a-robin-hobb-rereading-series-entry-7-assassins-apprentice-chapter-7/).

### Nick's vibe games: Expedition 33, The Witcher 3, Persona, Yakuza

- **One loss, a different key for each voice.** Expedition 33's party is
  told apart by how each carries loss: Sciel treats death like an old
  friend, Lune buries it in duty, Gustave jokes at himself. Its writer,
  Jennifer Svedberg-Yen, writes a moment by asking how that character, not
  she, would show the feeling.
- **The joke is the scar.** Sciel's jokes about water in Act I are paid off
  much later by the reason behind them. *Infinite Wealth*'s "Let It Snow"
  side story turns a man throwing ice in shops into grief, then back into a
  laugh.
- **Plant a promise; let an object close it.** Gustave and Maelle agree to
  run if the white-haired man comes back; he stays. Verso brings back his
  arm, his converter and his journal.
- **Sympathy first, sin second.** The Bloody Baron is first hospitable and
  loving; the beatings come out later. Geralt answers his long account
  with short stances the player picks.
- **Suppressed, not empty.** Doug Cockle recalls the early direction for
  Geralt: "No emotion, Doug. Flatten him out." He played a man holding his
  feelings down instead, and the writing shifts Geralt's register with the
  listener.
- **The cast completes the silent lead.** Persona's casts make the lead
  present by deferring to him and reacting to his odd replies; Kiryu's
  complete seriousness inside absurd side stories is the joke.
- **Avoid:** a cast praising a lead who barely speaks (Persona); hours of
  jokes right after a massacre (a critic of Expedition 33); Geralt's
  needless rudeness to strangers.

Sources: [Svedberg-Yen interview](https://www.invenglobal.com/articles/20217/clair-obscur-expedition-33-writer-jennifer-svedberg-yen-reveals-how-real-life-experiences-shaped-the-award-winning-story),
[Game Developer podcast](https://gamedeveloper.com/design/crafting-clair-obscur-expedition-33-s-mournful-tale-ft-jennifer-svedeberg-yen),
[Kotaku on the Bloody Baron](https://kotaku.com/the-story-behind-the-witcher-3s-bloody-baron-quest-1736090893),
[Doug Cockle on Geralt](https://exputer.com/news/games/geralts-voice-actor-emotionless-direction/),
[Hashino on Persona 5](https://gameinformer.com/b/features/archive/2015/12/09/persona-5-story-and-characters-interview-katsura-hashino),
[Infinite Wealth's side stories](https://butwhytho.net/2024/01/infinite-wealth-follows-up-on-like-a-dragons-best-substories).

## How long scenes run

Counted from transcripts by the research agents (one box here is up to
200 characters; the original games' boxes are smaller, so speaker turns
were counted and long turns split):

| Game | Scene | Length |
| ---- | ----- | ------ |
| FE7 | Prologue, before the first fight | 4 turns, about 6 boxes, under a minute |
| FE7 | Chapter 1, before the battle (two scenes) | 42 turns, about 2,500 characters |
| PoR | Chapter 1, before the battle (three short scenes) | about 30 turns, about 2,900 characters |
| Sacred Stones | Prologue (its first line: the gate is breached) | 33 turns, about 37 boxes |
| Three Houses | Prologue, before the first battle | about 33 boxes |
| FFT | Most Chapter 1 battles, before play | 2–9 lines (Dorter 14, the prologue 23) |
| Tactics Ogre | Prologue, before the first battle | about 12 speech panels |
| Triangle Strategy | The opening | one battle in about two and a half hours (MobileSyrup) |
| Unicorn Overlord | Any conversation | "no longer than a minute or two" (Nintendo Life) |
| Fire Emblem | Boss challenge line | 1 box, median 72 characters (17 measured) |
| Fire Emblem | Boss defeat line | 1 box, median 47 characters (19 measured) |
| Fire Emblem | A lord's special talk with a boss | median 5 turns (14 measured) |
| Fire Emblem | Ally death or retreat line | median 53 characters (40 measured) |
| Hades | A voiced line | about 70–85 characters on average (worked out from Supergiant's line and word counts) |
| **Ours** | **Chapter 1, before the first move** | **38–41 boxes** |

A full 200-character box takes about 8–10 seconds to read. The budget in
the skill follows from these, plus about two boxes a scene for the people
(round 2): a chapter's first scene of 8–14 boxes, a pre-battle scene of
5–10, all talk before the first move capped at 28, one box for a battle
line, a victory scene of 6–12. They are Claude's starting numbers, to be
tuned when Nick plays the result. The drag in Chapter 1 came from repeats
and reports (the seal worked out four or five times), not from warmth.

## What went into the skill

| Skill rule | Where it comes from |
| ---------- | ------------------- |
| Likeable people first; would a player miss them | Nick's round 2 answer; Bopp and others on worry versus admiration |
| Rewrites start on a blank page; quoted old lines count | 0724 and 0728 (above); Tian et al. on planning turning points first; humanizer and EQ-Bench (rewrite whole, never splice) |
| The scene card (feelings about each other, the body, one non-mission thought, who isn't okay, turn, start and cut, the obvious staging) | Mamet; round 2's critic; the ai-robots research (one thing that isn't the mission); EQ-Bench Longform's plan critique |
| Every scene has people being people | Gottman's bids; RDR2's camp; PoR's base talks; round 2's critic |
| Care lands on the body | Band of Brothers; Tolkien; O'Brian's Killick; PoR Ch8; invisible-support studies |
| Small feelings out loud, the big one once | StoryScope's 29% vs 8%; RDR2 (Arthur); PoR (Mist, Greil); M\*A\*S\*H's toast |
| Somebody isn't okay, somebody is wrong | Yi et al.; Tian et al.; Generative Agents; Mordin; Lenny |
| Small talk while the big thing happens; reach and miss | Chekhov; Lonergan; Annie Baker; Sacks |
| Disagree like people who like each other | Pomerantz; round 2's critic |
| Shared history as habit | Aubrey and Maturin; Sharpe and Harper; Tannen; FFT's grass whistles |
| Facts said once, by someone who cares; the map holds the numbers | McKee; Unicorn Overlord's map rule; PoR's one-sentence briefing |
| The plainest line is remembered; one punchy box a scene at most | round 2's critic; the tactics-death scripts (names, dots, pleas); Abercrombie's test |
| Plant something small for everyone who can die | The Blazing Blade (Matthew and Leila); PoR (Greil's night walk); RDR2; The Things They Carried |
| Grief and death: last words about the living, the one hurting says least, a small true story | PoR Ch7; The Blazing Blade Ch19; Three Houses Ch9; Tim O'Brien; M\*A\*S\*H |
| Narration shows people, never ends on a symbol | round 2's critic (the tally sticks); round 1's critic (narration doing the characters' work) |
| Voice cards with age, temperature, openness, fear, care signature, fault | round 2's critic; Nicklin; McKee; Three Houses' measured line lengths |
| Battle lines are one box; bosses with a grievance; frightened when dying | FE counts; FFT's triggers; Octopath's Gaston; Yi et al. |
| The silent lead: replies move someone; the cast notices the lead | Three Houses (Father's Diary choices); PoR (Greil); Thane; Valve; Persona's flattery problem |
| Getting human lines out of a model (five versions; improv with private wants) | Verbalized sampling; CoSER; Bowman and Fish; Generative Agents |
| Scene-length budget | the counts above; Nick's "drags in the conversations" |
| The cold read (who would you miss) and the editor's list (robots, the writer talking, syrup) | Anthropic's reader test; haowjy's reader simulation; EQ-Bench (faults kept out of the writer's prompt); round 2's critic |
| Plain-prose rules for narration only | stop-slop and humanizer's own scope; the *Antislop* paper |

## Round 2 sources

Scripts and scenes: Serenes Forest's Path of Radiance scripts
([Ch7](https://serenesforest.net/path-of-radiance/scripts/game-script/chapter-7-shades-of-evil/),
[Ch8](https://serenesforest.net/path-of-radiance/scripts/game-script/chapter-8-despair-and-hope/),
[Ch9](https://serenesforest.net/path-of-radiance/scripts/game-script/chapter-9-gallia/))
and The Blazing Blade's
[Ch17](https://serenesforest.net/blazing-sword/scripts/game-script/hectors-story/chapter-17-whereabouts-unknown/)
and [Ch19](https://serenesforest.net/blazing-sword/scripts/game-script/hectors-story/chapter-19-the-dread-isle/);
Three Houses' [Father's Diary](https://houses.fedatamine.com/en-us/scenarios/106)
and [Jeralt's last words](https://houses.fedatamine.com/en-us/battles/9);
the LP Archive's [Final Fantasy Tactics, Mandalia sunset](https://lparchive.org/Final-Fantasy-Tactics/Update%2010/);
[Naughty Dog's podcast on The Last of Us prologue](https://www.naughtydog.com/podcasts/thelastofus/en/ep-01);
[Wikiquote on Red Dead Redemption 2](https://en.wikiquote.org/wiki/Red_Dead_Redemption_2);
Chekhov's [The Cherry Orchard, Act IV](https://ibiblio.org/eldritch/ac/coa4.htm);
[Lonergan in Film Comment](https://www.filmcomment.com/interview-kenneth-lonergan-manchester-by-the-sea/);
[Kotaku on Firewatch](https://kotaku.com/a-video-game-about-how-people-actually-talk-to-each-oth-1629656934);
[LitCharts on The Things They Carried](https://www.litcharts.com/lit/the-things-they-carried/the-things-they-carried);
[A.V. Club on Band of Brothers, Bastogne](https://www.avclub.com/band-of-brothers-bastogne-1798180391);
[Abyssinia, Henry](https://en.wikipedia.org/wiki/Abyssinia,_Henry).

Research: [Gottman on bids](https://www.gottman.com/blog/turn-toward-instead-of-away/);
[Howland and Simpson on invisible support](https://socialinteractionlab.psych.umn.edu/sites/socialinteractionlab.psych.umn.edu/files/files/2010/2010/Howland%2520%2526%2520Simpson%2520%2528PS%252C%25202010%2529.pdf);
[Keltner and others on teasing](https://greatergood.berkeley.edu/dacherkeltner/docs/keltner.teasing.jpsp.1998.pdf);
[Pomerantz 1984](https://emcawiki.net/Pomerantz1984a); [Jefferson 1984](https://emcawiki.net/Jefferson1984a);
[Bopp and others on attachment to game characters](https://edoc.unibas.ch/73889);
[StoryScope](https://arxiv.org/html/2604.03136v6);
[The Assistant Axis](https://arxiv.org/abs/2601.10387);
[The Persona Selection Model](https://www.anthropic.com/research/persona-selection-model);
[Generative Agents](https://arxiv.org/abs/2304.03442);
[Too Good to be Bad](https://arxiv.org/abs/2511.04962);
[CoSER](https://arxiv.org/abs/2502.09082);
[Verbalized Sampling](https://arxiv.org/abs/2510.01171);
[Bowman and Fish in Asterisk](https://asteriskmag.com/issues/11/claude-finds-god).

Models: [EQ-Bench Creative Writing](https://eqbench.com/creative_writing.html),
[EQ-Bench Longform](https://eqbench.com/creative_writing_longform.html),
[Lech Mazur's benchmark](https://github.com/lechmazur/writing),
[Anthropic's Commercial Terms](https://www.anthropic.com/legal/commercial-terms),
[Chakrabarty and others, 2026](https://arxiv.org/abs/2510.13939).

## Quotes kept for study

Each under 15 words, credited, here only:

- "Ask yourself with every line of dialogue, would someone really say
  this." Joe Abercrombie, [interview](https://authorinterviews.co.uk/?p=416).
- "If it sounds like writing, I rewrite it." Elmore Leonard, 10 Rules of
  Writing (NYT, 2001).
- "It is not really simple, but flat." Ursula K. Le Guin, "From Elfland to
  Poughkeepsie" (1973).
- "disruptive almost all the time" (thee and thou). Alexander O. Smith,
  Game Developer interview (2011).
- "No emotion, Doug. Flatten him out." Doug Cockle recalling the early
  direction for Geralt (eXputer).
- "excruciatingly, painfully, polite at all times". IGN's Triangle
  Strategy review, on its four leads.
- "Waited too long, didn't I?" Matthew, Fire Emblem: The Blazing Blade,
  Hector's Ch19, after Leila's death (Serenes Forest script).
- "it's not much, but you have me" Robert Kurvitz on Kim Kitsuragi
  (via Wikipedia).
