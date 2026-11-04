#include <stdio.h>
#include <string.h>

#include "greeting.h"

int main(void) {
  if (strcmp(greeting(), "Hello, world!") != 0) {
    fprintf(stderr, "greeting() returned \"%s\"\n", greeting());
    return 1;
  }
  return 0;
}
