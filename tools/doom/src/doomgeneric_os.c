#include "doomgeneric/doomgeneric.h"
#include "doomgeneric/doomkeys.h"
#include <stdint.h>
#include <stddef.h>

// Pointeur vers l'API kernel passé par _start
typedef struct {
    void (*draw_frame)(const uint32_t* pixels, uint32_t width, uint32_t height);
    int  (*get_key)(int* pressed, unsigned char* key);
    uint32_t (*get_ticks)(void);
} KernelApi;

static const KernelApi* g_api = NULL;

// Key queue
#define KEYQUEUE_SIZE 16
static unsigned short s_KeyQueue[KEYQUEUE_SIZE];
static unsigned int s_KeyQueueWriteIndex = 0;
static unsigned int s_KeyQueueReadIndex  = 0;

void DG_Init() {}

void DG_DrawFrame() {
    if (g_api && DG_ScreenBuffer) {
        g_api->draw_frame(DG_ScreenBuffer, DOOMGENERIC_RESX, DOOMGENERIC_RESY);
    }
}

void DG_SleepMs(uint32_t ms) {
    // no-op — kernel gère le timing
}

uint32_t DG_GetTicksMs() {
    if (g_api) return g_api->get_ticks();
    return 0;
}

int DG_GetKey(int* pressed, unsigned char* doomKey) {
    if (s_KeyQueueReadIndex == s_KeyQueueWriteIndex) return 0;

    unsigned short keyData = s_KeyQueue[s_KeyQueueReadIndex];
    s_KeyQueueReadIndex = (s_KeyQueueReadIndex + 1) % KEYQUEUE_SIZE;

    *pressed = keyData >> 8;
    *doomKey = keyData & 0xFF;
    return 1;
}

void DG_SetWindowTitle(const char* title) {}

// Point d'entrée appelé par le kernel
void doom_os_init(const KernelApi* api) {
    g_api = api;
    
    char* argv[] = { "doom", NULL };
    doomgeneric_Create(1, argv);
}

void doom_os_tick(void) {
    doomgeneric_Tick();
}

// Pour ajouter des touches depuis le kernel plus tard
void doom_os_add_key(int pressed, unsigned char key) {
    unsigned short keyData = (pressed << 8) | key;
    s_KeyQueue[s_KeyQueueWriteIndex] = keyData;
    s_KeyQueueWriteIndex = (s_KeyQueueWriteIndex + 1) % KEYQUEUE_SIZE;
}






// ── stubs ──────────────────────────────────────────────────────────────────

#include <stdarg.h>

typedef void FILE;
FILE* stderr = (FILE*)2;
FILE* stdout = (FILE*)1;

int    fclose(FILE* s)                                { return 0; }
size_t fwrite(const void* p, size_t s, size_t n, FILE* f) { return n; }
int    fseek(FILE* s, long o, int w)                  { return 0; }
long   ftell(FILE* s)                                 { return 0; }
int    ferror(FILE* s)                                { return 0; }
int    fflush(FILE* s)                                { return 0; }
int    remove(const char* p)                          { return 0; }
int    rename(const char* o, const char* n)           { return 0; }

int    puts(const char* s)                            { return 0; }
int    putchar(int c)                                 { return c; }
int    putc(int c, FILE* f)                           { return c; }
int    printf(const char* fmt, ...)                   { return 0; }
int    fprintf(FILE* f, const char* fmt, ...)         { return 0; }
int    sprintf(char* s, const char* fmt, ...)         { return 0; }
int    snprintf(char* s, size_t n, const char* fmt, ...){ if(n>0) s[0]=0; return 0; }
int    vsnprintf(char* s, size_t n, const char* fmt, va_list ap){ if(n>0) s[0]=0; return 0; }
int    vfprintf(FILE* f, const char* fmt, va_list ap) { return 0; }
int    sscanf(const char* s, const char* fmt, ...)    { return 0; }

int    toupper(int c) { return (c>='a'&&c<='z') ? c-'a'+'A' : c; }

size_t strlen(const char* s)  { size_t n=0; while(s[n]) n++; return n; }
char*  strncpy(char* d, const char* s, size_t n) {
    size_t i=0;
    for(;i<n&&s[i];i++) d[i]=s[i];
    for(;i<n;i++) d[i]=0;
    return d;
}
int strcmp(const char* a, const char* b) {
    while(*a&&*a==*b){a++;b++;}
    return (unsigned char)*a-(unsigned char)*b;
}
int strncmp(const char* a, const char* b, size_t n) {
    for(size_t i=0;i<n;i++){
        if(a[i]!=b[i]||(a[i]==0)) return (unsigned char)a[i]-(unsigned char)b[i];
    }
    return 0;
}
int strcasecmp(const char* a, const char* b) {
    while(*a&&toupper(*a)==toupper(*b)){a++;b++;}
    return toupper(*a)-toupper(*b);
}
int strncasecmp(const char* a, const char* b, size_t n) {
    for(size_t i=0;i<n;i++){
        int ca=toupper(a[i]),cb=toupper(b[i]);
        if(ca!=cb||ca==0) return ca-cb;
    }
    return 0;
}
char* strchr(const char* s, int c) {
    while(*s){ if(*s==(char)c) return (char*)s; s++; }
    return NULL;
}
char* strrchr(const char* s, int c) {
    char* last=NULL;
    while(*s){ if(*s==(char)c) last=(char*)s; s++; }
    return last;
}
int atoi(const char* s) {
    int r=0,sign=1;
    while(*s==' '||(*s>=9&&*s<=13)) s++;
    if(*s=='-'){sign=-1;s++;} else if(*s=='+') s++;
    while(*s>='0'&&*s<='9') r=r*10+(*s++-'0');
    return sign*r;
}

static char strdup_arena[16384];
static size_t strdup_offset = 0;
char* strdup(const char* s) {
    size_t len = strlen(s) + 1;
    if (strdup_offset + len > sizeof(strdup_arena)) return NULL;
    char* dst = strdup_arena + strdup_offset;
    strdup_offset += len;
    for (size_t i = 0; i < len; i++) dst[i] = s[i];
    return dst;
}

char* getenv(const char* n)  { return NULL; }
int   system(const char* c)  { return 0; }
void  exit(int s)             { while(1); }
void  abort(void)             { while(1); }
int   mkdir(const char* p, unsigned int m) { return 0; }

void __stack_chk_fail(void)   { while(1); }

int global_errno = 0;
int* __errno_location(void)   { return &global_errno; }

static unsigned short ctype_table[384];
static int ctype_init = 0;
const unsigned short** __ctype_b_loc(void) {
    static const unsigned short* p = NULL;
    if (!ctype_init) {
        ctype_init = 1;
        p = &ctype_table[128];
        ctype_table[128+' '] = 0x2000;
        ctype_table[128+'\t']= 0x2000;
        ctype_table[128+'\n']= 0x2000;
        ctype_table[128+'\r']= 0x2000;
        for(int i='0';i<='9';i++) ctype_table[128+i]=0x0800;
        for(int i='a';i<='z';i++) ctype_table[128+i]=0x0400;
        for(int i='A';i<='Z';i++) ctype_table[128+i]=0x0200;
    }
    return &p;
}

static int toupper_table[384];
static int toupper_init = 0;
const int** __ctype_toupper_loc(void) {
    static const int* p = NULL;
    if (!toupper_init) {
        toupper_init = 1;
        for(int i=0;i<384;i++){
            int c=(i-128);
            toupper_table[i]=(c>='a'&&c<='z') ? c-('a'-'A') : c;
        }
        p = &toupper_table[128];
    }
    return &p;
}

int __isoc23_sscanf(const char* s, const char* fmt, ...) { return 0; }
long __isoc23_strtol(const char* s, char** end, int base) {
    long r=0; int neg=0;
    while(*s==' ') s++;
    if(*s=='-'){neg=1;s++;} else if(*s=='+') s++;
    if(base==16&&s[0]=='0'&&(s[1]=='x'||s[1]=='X')) s+=2;
    while(1){
        int d;
        if(*s>='0'&&*s<='9') d=*s-'0';
        else if(*s>='a'&&*s<='z') d=*s-'a'+10;
        else if(*s>='A'&&*s<='Z') d=*s-'A'+10;
        else break;
        if(d>=base) break;
        r=r*base+d; s++;
    }
    if(end) *end=(char*)s;
    return neg?-r:r;
}

void __printf_chk(int f, const char* fmt, ...)         {}
void __fprintf_chk(FILE* s, int f, const char* fmt, ...) {}
void __vfprintf_chk(FILE* s, int f, const char* fmt, va_list ap) {}
int  __snprintf_chk(char* s, size_t n, int f, size_t dn, const char* fmt, ...) { if(n>0) s[0]=0; return 0; }
int  __vsnprintf_chk(char* s, size_t n, int f, size_t dn, const char* fmt, va_list ap) { if(n>0) s[0]=0; return 0; }
void* __memset_chk(void* d, int c, size_t n, size_t dn) { 
    unsigned char* p=d; for(size_t i=0;i<n;i++) p[i]=c; return d; 
}
void* __memcpy_chk(void* d, const void* s, size_t n, size_t dn) {
    unsigned char* dp=d; const unsigned char* sp=s;
    for(size_t i=0;i<n;i++) dp[i]=sp[i]; return d;
}
char* __strncpy_chk(char* d, const char* s, size_t n, size_t dn) { return strncpy(d,s,n); }

int drone = 0;
int net_client_connected = 0;