// mod_file_io.cpp : �g�ݍ��݊֐��Ffile�A�N�Z�X�֌W
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"



// SHIORI ���X�|���X������Ԃ��܂��B
static duk_ret_t response(duk_context *ctx){
    USES_CONVERSION;
    OutputDebugString(L"[pasta::Shiori::response]�J�n�I\n");

    // �������������擾
    auto ghost = pasta::GetPasta(ctx);
    auto res = A2CW_UTF8(duk_to_string(ctx, 0));

    ghost->Response(res);

    return 1;
}


//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "response", response, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitShiori(){
    duk_push_global_object(ctx);
    duk_push_object(ctx);
    duk_put_function_list(ctx, -1, funcs);
    duk_put_prop_string(ctx, -2, "Shiori");
    duk_pop(ctx);
}

// EOF