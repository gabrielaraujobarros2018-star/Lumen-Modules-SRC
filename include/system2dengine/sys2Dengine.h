#ifndef SYS2DENGINE_H
#define SYS2DENGINE_H

// === LUMEN OS 2D Graphics Engine v1.0 ===
// ARMv7a optimized for Moto Nexus 6 (1440x2560@60Hz)
// Features: Glassmorphism, NEON accel, VSYNC, audio, chaos events, freeze recovery

#include <stdint.h>

// Core types
typedef uint32_t color_t;
typedef struct { int32_t x, y; } point_t;
typedef struct { int32_t x, y, w, h; } rect_t;

// Engine initialization
int sys2d_init(void);
void sys2d_shutdown(void);
void sys2d_render_sync(void);  // Main 60FPS VSYNC render loop

// Layers & Compositing
int sys2d_create_layer(rect_t bounds, int layer_id);
void sys2d_set_layer_dirty(int layer_id);

// Glassmorphism Effects
int sys_glass_create_layer(rect_t bounds, float corner_radius, uint8_t blur_radius);
void sys_glass_set_dirty(int layer_id);
void sys_glass_add_glow(point_t pos, int size, color_t color);
void sys_glass_demo(void);

// Audio System
int sys_audio_init(void);
void sys_audio_play_boop(void);
void sys_audio_play_click(void);
void sys_audio_beep(float freq, float duration, float volume);

// Funny Events & Chaos
void sys2d_set_funny_events(int enable);
void sys2d_enable_chaos_events(int enable);
void sys2d_force_funny_event(int event_id);
void sys2d_force_chaos_event(int event_id);

// Debug & Monitoring
void sys2d_set_fps_display(int enable);
void sys2d_toggle_fps_display(void);
void sys2d_debug_stats(void);
void sys2d_neon_stats(void);
void sys2d_force_freeze_recovery(void);

// Input
int sys2d_poll_input(point_t* touch_pos, int* buttons);

// Production flags
#define SYS2D_VERSION "1.0"
#define NEXUS6_SCREEN_WIDTH 1440
#define NEXUS6_SCREEN_HEIGHT 2560

#endif // SYS2DENGINE_H
