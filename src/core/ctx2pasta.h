#pragma once

#include "agent_pasta.h"
#include <duktape.h>

namespace pasta{
    // duk_context�ɑΉ�����pasta�G�[�W�F���g�ւ̎Q�Ƃ�o�^����B
    void SetPasta(duk_context* ctx, pasta::Agent* pasta);

    // duk_context�ɑΉ�����pasta�G�[�W�F���g�ւ̎Q�Ƃ��擾����B
    Agent* GetPasta(duk_context* ctx);
}