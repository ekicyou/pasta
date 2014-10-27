// mod_file_io.cpp : �g�ݍ��݊֐��Ffile�A�N�Z�X�֌W
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"

static wchar_t * preLoadPath[] = {
    L"duktape",
    L"modules",
    L"lib",
    L"js",
    L".",
    NULL,
};

// ���W���[���t�@�C�����������čŏ��Ɍ����������̂�ǂݍ��݃I�[�v�����ĕԂ��B
static FILE* openModuleFile(const pasta::Agent* pasta, LPCWSTR fname){
    if (!fname) return NULL;
    const auto &loaddir = pasta->loaddir;
    for (int i = 0;; i++){
        const auto pre = preLoadPath[i];
        if (!pre) return NULL;
        auto p = std::tr2::sys::wpath(pasta->loaddir);
        p /= pre;
        p /= fname;

        // �t�@�C�����J��
        auto f = _wfopen(p.string().c_str(), L"rb");
        if (f) return f;
    }
}

// �t�@�C�����o�b�t�@�Ƃ��ēǂݍ��݂܂��B
static duk_ret_t readfile(duk_context *ctx){
    USES_CONVERSION;
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // �������������擾
    auto fname = A2CW_UTF8(duk_to_string(ctx, 0));

    // �t�@�C���I�[�v��
    auto f = openModuleFile(pasta, fname);
    if (!f)goto error;

    // �ǂݍ���
    if (fseek(f, 0, SEEK_END) != 0) goto error;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto error;
    auto buf = duk_push_fixed_buffer(ctx, (size_t)len);
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len) goto error;

    if (f) fclose(f);
    return 1;

error:
    if (f) fclose(f);
    return DUK_RET_ERROR;
}

// �t�@�C�����e�L�X�g�Ƃ��ēǂݍ��݂܂��B
static duk_ret_t readtext(duk_context *ctx){
    USES_CONVERSION;
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // �������������擾
    auto fname = A2CW_UTF8(duk_to_string(ctx, 0));
    char *buf = NULL;

    // �t�@�C���I�[�v��
    auto f = openModuleFile(pasta, fname);
    if (!f)goto error;

    // �ǂݍ���
    if (fseek(f, 0, SEEK_END) != 0) goto error;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto error;
    buf = (char *)malloc(len);
    if (!buf)                       goto error;
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len)         goto error;
    duk_push_lstring(ctx, buf, got);

cleanup:
    free(buf);
    fclose(f);
    return 1;

error:
    if (buf) free(buf);
    if (f)   fclose(f);
    return DUK_RET_ERROR;
}

//-------------------------------------------------------------
// �f�o�b�O�o��
//-------------------------------------------------------------
static duk_ret_t debugstring(duk_context *ctx){
    // �������������擾
    auto ghost = pasta::GetPasta(ctx);
    auto cp = ghost->cp;
#ifdef DEBUG
    std::string text(duk_to_string(ctx, 0));
    text += "\n";
    USES_CONVERSION;
    OutputDebugString(A2CW_CP(text.c_str(), cp));
#endif

    return 1;
}


//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "readfile"    , readfile      , 1 },
        { "readtext"    , readtext      , 1 },
        { "debugstring" , debugstring   , 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitFileIO(){
    RegModuleFuncs("FileIO", funcs);
}

// EOF