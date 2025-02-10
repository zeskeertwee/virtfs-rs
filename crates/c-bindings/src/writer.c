#include "../bindings.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

void callback(const char *id, uintptr_t id_len, const char *data, uintptr_t len,
              uint64_t location) {
  printf("Processed Leaf: ID=%s, Len=%llu, Location=%llu\n", id, len, location);
}

int main(int argc, char **argv) {
  if (argc < 2) {
    printf("[No path provided] Usage: %s <out_file> [in files]\n", argv[0]);
    return 1;
  }

  // get output path
  char *out_path = argv[1];
  int32_t error_p = 0;

  // allocate space for leaves, one leaf per file
  int paths_count = argc - 2;
  v_builder_ctx *ctx = new_builder_ctx(NULL, NULL, 0);

  // initialize leaves
  char *id_buf = calloc(256, sizeof(char));

  for (int i = 0; i < paths_count; i++) {
    char *path = argv[i + 2];
    snprintf(id_buf, 256 * sizeof(char), "ID[%d]=%s", i, path);

    error_p = 0;
    add_leaf_from_file(ctx, id_buf, path, 0, &error_p);

    if (error_p != 0) {
      printf("Unable to process Leaf: %s, Error = %d\n", id_buf, error_p);
      continue;
    }
  }

  free(id_buf);

  // process leaves and write archive
  uintptr_t bytes = dump_leaves_to_file(ctx, out_path, &callback, &error_p);
  printf("Processed Archived: Path=%s, Bytes=%llu\n", out_path, bytes);
}