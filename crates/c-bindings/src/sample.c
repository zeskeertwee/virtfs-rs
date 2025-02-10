#include "../bindings.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(int argc, char **argv) {
  if (argc < 2) {
    printf("[No path provided] Usage: %s <path>\n", argv[0]);
    return 1;
  }

  // get archive path
  char *path = argv[1];
  int32_t error_p = 0;

  // use default archive_config
  v_archive *archive = new_archive_from_file(path, NULL, &error_p);
  if (error_p != 0) {
    printf("Error: %d\n", error_p);
  }

  // get entries
  v_entries *entries = archive_get_entries(archive, &error_p);
  if (error_p != 0) {
    printf("Error: %d\n", error_p);
  }

  // load resources
  for (int i = 0; i < entries->count; i++) {
    char *path = entries->paths[i];
    v_resource *resource = archive_fetch_resource(archive, path, &error_p);

    // log v_resource data
    printf("[%d] Path: %s, Len: %llu, Flags: %d\n", i, path, resource->len,
           resource->flags);

    // free resource
    free_resource(resource);
  }

  // free memory
  free_entries(entries);
  free_archive(archive);
}