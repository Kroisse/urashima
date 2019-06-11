#ifndef _naru__bindings_h_
#define _naru__bindings_h_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct NaruRuntime NaruRuntime;

void naru_runtime_delete(NaruRuntime *rt);

void naru_runtime_execute(NaruRuntime *rt, const char *path);

const Error *naru_runtime_last_error(NaruRuntime *rt);

NaruRuntime *naru_runtime_new(void);

#endif /* _naru__bindings_h_ */
