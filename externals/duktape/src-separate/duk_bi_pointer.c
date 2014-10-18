/*
 *  Pointer built-ins
 */

#include "duk_internal.h"

/*
 *  Constructor
 */

duk_ret_t duk_bi_pointer_constructor(duk_context *ctx) {
	/* XXX: this behavior is quite useless now; it would be nice to be able
	 * to create pointer values from e.g. numbers or strings.  Numbers are
	 * problematic on 64-bit platforms though.  Hex encoded strings?
	 */
	if (duk_get_top(ctx) == 0) {
		duk_push_pointer(ctx, NULL);
	}
	else {
		duk_to_pointer(ctx, 0);
	}
	DUK_ASSERT(duk_is_pointer(ctx, 0));
	duk_set_top(ctx, 1);

	if (duk_is_constructor_call(ctx)) {
		duk_push_object_helper(ctx,
			DUK_HOBJECT_FLAG_EXTENSIBLE |
			DUK_HOBJECT_CLASS_AS_FLAGS(DUK_HOBJECT_CLASS_POINTER),
			DUK_BIDX_POINTER_PROTOTYPE);

		/* Pointer object internal value is immutable */
		duk_dup(ctx, 0);
		duk_def_prop_stridx(ctx, -2, DUK_STRIDX_INT_VALUE, DUK_PROPDESC_FLAGS_NONE);
	}
	/* Note: unbalanced stack on purpose */

	return 1;
}

/*
 *  toString(), valueOf()
 */

duk_ret_t duk_bi_pointer_prototype_tostring_shared(duk_context *ctx) {
	duk_tval *tv;
	duk_small_int_t to_string = duk_get_current_magic(ctx);

	duk_push_this(ctx);
	tv = duk_require_tval(ctx, -1);
	DUK_ASSERT(tv != NULL);

	if (DUK_TVAL_IS_POINTER(tv)) {
		/* nop */
	}
	else if (DUK_TVAL_IS_OBJECT(tv)) {
		duk_hobject *h = DUK_TVAL_GET_OBJECT(tv);
		DUK_ASSERT(h != NULL);

		/* Must be a "pointer object", i.e. class "Pointer" */
		if (DUK_HOBJECT_GET_CLASS_NUMBER(h) != DUK_HOBJECT_CLASS_POINTER) {
			goto type_error;
		}

		duk_get_prop_stridx(ctx, -1, DUK_STRIDX_INT_VALUE);
	}
	else {
		goto type_error;
	}

	if (to_string) {
		duk_to_string(ctx, -1);
	}
	return 1;

type_error:
	return DUK_RET_TYPE_ERROR;
}