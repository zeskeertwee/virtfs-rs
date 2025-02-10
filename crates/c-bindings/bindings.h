#pragma once

#include <stdbool.h>
#include <stdint.h>

// The length of a public key
#define V_VERIFYING_KEY_LENGTH 32

// The length of a secret
#define V_SECRET_KEY_LENGTH 32

// One parameter passed to a function was NULL
#define E_PARAMETER_IS_NULL -1

// Unable to parse a key or signature
#define E_PARSE_ERROR -2

// String parameter was not valid UTF8 sequence
#define E_INVALID_UTF8 -3

// Generic IO error
#define E_GENERIC_IO_ERROR -4

// Malformed archive source, invalid MAGIC or otherwise
#define E_MALFORMED_ARCHIVE_SOURCE -5

// Resource not found
#define E_RESOURCE_NOT_FOUND -6

// Unknown error
#define E_UNKNOWN -7

// One or more necessary library features wasn't enabled during compilation
#define E_MISSING_FEATURE_ERROR -8

// Generic cryptographic error, signature verification failed or otherwise
#define E_CRYPTO_ERROR -9

// Generic cryptographic error, signature verification failed or otherwise
#define E_LEAF_ID_TOO_LONG -10

// Verifying and Decryption Key
typedef void v_verifying_key;

// An Archive instance, bound to either a file or a buffer
typedef void v_archive;

// A list archive entry IDs
typedef struct v_entries {
  uintptr_t count;
  char **paths;
} v_entries;

// An archive resource
typedef struct v_resource {
  uint8_t *data;
  uintptr_t len;
  unsigned int flags;
  uint8_t content_version;
  bool verified;
} v_resource;

// Archive Builder Context
typedef void v_builder_ctx;

// Archive Builder Configuration, use `libffcall` to construct closures in C
typedef void (*v_builder_callback)(const char *id, uintptr_t id_len, const char *data, uintptr_t len, uint64_t location);

// The version of the library
uint16_t version(void);

// Create new loader configuration
v_verifying_key *new_verifying_key(const uint8_t (*vk_bytes)[V_VERIFYING_KEY_LENGTH], int32_t *error_p);

// Free archive loader configuration
void free_verifying_key(v_verifying_key *config);

// Create a new archive from a file
v_archive *new_archive_from_file(const char *path, const v_verifying_key *config, int32_t *error_p);

// Create a new archive from a buffer
v_archive *new_archive_from_buffer(const v_verifying_key *config, const uint8_t *data, uintptr_t len, int32_t *error_p);

void free_archive(v_archive *archive);

// Get a list of archive entry IDs
struct v_entries *archive_get_entries(const v_archive *archive, int32_t *error_p);

void free_entries(struct v_entries *entries);

// Fetch a resource, WITHOUT locking the internal Mutex
struct v_resource *archive_fetch_resource(v_archive *archive, const char *id, int32_t *error_p);

// Fetch a resource, LOCKS the internal Mutex. For use in multithreaded environments
struct v_resource *archive_fetch_resource_lock(const v_archive *archive, const int8_t *id, int32_t *error_p);

void free_resource(struct v_resource *resource);

// Create new Builder Context
v_builder_ctx *new_builder_ctx(const uint8_t (*sk_bytes)[V_SECRET_KEY_LENGTH], uint32_t flags);

// free memory bound by `new_builder_ctx`
void free_builder_ctx(v_builder_ctx *ctx);

// Appends a new `v_builder_leaf` from a buffer
void add_leaf_from_buffer(v_builder_ctx *ctx, const char *id, const uint8_t *data, uintptr_t len, uint32_t flags, int32_t *error_p);

// Creates a new `v_builder_leaf` from a file
void add_leaf_from_file(v_builder_ctx *ctx, const char *id, const char *path, uint32_t flags, int32_t *error_p);

// process context and dump to a preallocated buffer, buffer must at least be big enough to fit data
uint64_t dump_archive_to_buffer(v_builder_ctx *ctx, uint8_t *buffer, uintptr_t buf_size, v_builder_callback callback, int32_t *error_p);

// processed context and write to a file on disk
uint64_t dump_leaves_to_file(v_builder_ctx *ctx, const char *path, v_builder_callback callback, int32_t *error_p);
