// stdafx.h : �W���̃V�X�e�� �C���N���[�h �t�@�C���̃C���N���[�h �t�@�C���A�܂���
// �Q�Ɖ񐔂������A�����܂�ύX����Ȃ��A�v���W�F�N�g��p�̃C���N���[�h �t�@�C��
// ���L�q���܂��B
//

#pragma once

#include "targetver.h"

#define WIN32_LEAN_AND_MEAN             // Windows �w�b�_�[����g�p����Ă��Ȃ����������O���܂��B
// Windows �w�b�_�[ �t�@�C��:
#include <windows.h>


#define _ATL_CSTRING_EXPLICIT_CONSTRUCTORS      // �ꕔ�� CString �R���X�g���N�^�[�͖����I�ł��B

#include <atlbase.h>
#include <atlstr.h>

// TODO: �v���O�����ɕK�v�Ȓǉ��w�b�_�[�������ŎQ�Ƃ��Ă��������B

// PSL Setting  (https://github.com/Silica/PSL/wiki/compile-option)
#define PSL_OPTIMIZE_TAILCALL	// �����Ăяo���œK��
#define PSL_OPTIMIZE_IN_COMPILE	// �R���p�C�����œK��(����ȉ�)��L���ɂ���
#define PSL_THREAD_SAFE	// ��̃I�v�V�����𖳌��ɂ��A���[�J��static�ϐ����g��Ȃ��Ȃ�

#include "..\\..\\..\\..\\PSL\\PSL\\PSL.h"
