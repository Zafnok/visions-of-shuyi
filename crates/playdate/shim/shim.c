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

// --- buttons ----------------------------------------------------------------

// `src/pads.rs` keeps a copy of these bits; the compiler checks it here.
_Static_assert(kButtonLeft == 1 && kButtonRight == 2 && kButtonUp == 4 && kButtonDown == 8
		&& kButtonB == 16 && kButtonA == 32,
	"the button bits in src/pads.rs no longer match pd_api_sys.h");

// The buttons held now and the ones pressed since the last call.
void pd_buttons(unsigned int* current, unsigned int* pushed)
{
	PDButtons held, down, up;
	pd->system->getButtonState(&held, &down, &up);
	*current = (unsigned int)held;
	*pushed = (unsigned int)down;
}

// --- the display ----------------------------------------------------------------

// `src/pd/screen.rs` writes the frame buffer as `trpg_ui::onebit` lays it out.
_Static_assert(LCD_ROWSIZE == 52 && LCD_ROWS == 240,
	"the frame buffer is no longer 52 bytes by 240 rows (trpg_ui::onebit)");

void pd_set_refresh_rate(float rate) { pd->display->setRefreshRate(rate); }
unsigned char* pd_frame(void) { return pd->graphics->getFrame(); }
void pd_mark_updated_rows(int start, int end) { pd->graphics->markUpdatedRows(start, end); }

// --- files (the game's data folder) -------------------------------------------

void* pd_file_open_write(const char* name) { return pd->file->open(name, kFileWrite); }
// The data folder first, then the game's own `.pdx`.
void* pd_file_open_read(const char* name) { return pd->file->open(name, kFileReadData | kFileRead); }
// The data folder only.
void* pd_file_open_read_data(const char* name) { return pd->file->open(name, kFileReadData); }
int pd_file_read(void* file, void* buf, unsigned int len) { return pd->file->read((SDFile*)file, buf, len); }
int pd_file_write(void* file, const void* buf, unsigned int len)
{
	return pd->file->write((SDFile*)file, buf, len);
}
int pd_file_close(void* file) { return pd->file->close((SDFile*)file); }
const char* pd_file_error(void) { return pd->file->geterr(); }
// 1 if `name` is a file, 0 if it is a folder, -1 if there is nothing there.
int pd_file_kind(const char* name)
{
	FileStat stat;
	if (pd->file->stat(name, &stat) != 0) {
		return -1;
	}
	return stat.isdir ? 0 : 1;
}
int pd_file_list(const char* path, void (*each)(const char* name, void* userdata), void* userdata)
{
	return pd->file->listfiles(path, each, userdata, 0);
}
int pd_file_mkdir(const char* path) { return pd->file->mkdir(path); }
int pd_file_unlink(const char* name) { return pd->file->unlink(name, 0); }
int pd_file_rename(const char* from, const char* to) { return pd->file->rename(from, to); }

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
