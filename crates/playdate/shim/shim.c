// The C side of the Playdate front end (ADR-0070 §2). The SDK's own build
// support (setup.c, link_map.ld) does the platform part; this file is the
// SDK's entry point and one small wrapper per SDK call the Rust library
// (`trpg-playdate`, `src/pd/`) uses. It is compiled against the installed
// SDK's pd_api.h, so the C compiler checks every call against the real API.
// The SDK's headers are never copied into the repository.
#include "pd_api.h"

static PlaydateAPI* pd;

// The Rust side (`src/pd/entry.rs`).
extern void trpg_playdate_init(void);
extern int trpg_playdate_update(void);

// --- memory, console, errors ------------------------------------------------

void* pd_realloc(void* ptr, size_t size) { return pd->system->realloc(ptr, size); }
void pd_log(const char* text) { pd->system->logToConsole("%s", text); }
void pd_error(const char* text) { pd->system->error("%s", text); }

// --- time -------------------------------------------------------------------

unsigned int pd_millis(void) { return pd->system->getCurrentTimeMilliseconds(); }
void pd_reset_elapsed(void) { pd->system->resetElapsedTime(); }
float pd_elapsed(void) { return pd->system->getElapsedTime(); }

// --- drawing ----------------------------------------------------------------

void pd_clear(void) { pd->graphics->clear(kColorWhite); }
void pd_draw_text(const char* text, size_t len, int x, int y)
{
	pd->graphics->drawText(text, len, kUTF8Encoding, x, y);
}

// --- files (the game's data folder) -------------------------------------------

void* pd_file_open_write(const char* name) { return pd->file->open(name, kFileWrite); }
// The data folder first, then the game's own `.pdx`.
void* pd_file_open_read(const char* name) { return pd->file->open(name, kFileReadData | kFileRead); }
int pd_file_read(void* file, void* buf, unsigned int len) { return pd->file->read((SDFile*)file, buf, len); }
int pd_file_write(void* file, const void* buf, unsigned int len)
{
	return pd->file->write((SDFile*)file, buf, len);
}
int pd_file_close(void* file) { return pd->file->close((SDFile*)file); }

// --- the entry point ----------------------------------------------------------

static int update(void* userdata)
{
	(void)userdata;
	return trpg_playdate_update();
}

#ifdef _WINDLL
__declspec(dllexport)
#endif
int eventHandler(PlaydateAPI* playdate, PDSystemEvent event, uint32_t arg)
{
	(void)arg;
	if (event == kEventInit) {
		pd = playdate;
		trpg_playdate_init();
		pd->system->setUpdateCallback(update, NULL);
	}
	return 0;
}
