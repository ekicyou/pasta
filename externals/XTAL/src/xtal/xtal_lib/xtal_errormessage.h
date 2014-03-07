
#pragma once

namespace xtal{

inline void bind_error_message(){
	const char_t* messages[] = {
		XTAL_L("XCE1001"), XTAL_L("XCE1001:�\���G���[�ł��B"),
		XTAL_L("XCE1002"), XTAL_L("XCE1002:'%(required)s'�����҂��܂�����'%(char)s'�����o����܂����B"),
		XTAL_L("XCE1003"), XTAL_L("XCE1003:';' ������܂���B"),
		XTAL_L("XCE1004"), XTAL_L("XCE1004:�s���Ȋ֐��̉������ł��B"),
		XTAL_L("XCE1005"), XTAL_L("XCE1005:�񖼑O�t�����������O�t�������̌�ɂ���܂��B"),
		XTAL_L("XCE1006"), XTAL_L("XCE1006:�s����break���A�܂���continue���ł��B"),
		XTAL_L("XCE1008"), XTAL_L("XCE1008:�s���ȑ��d������ł��B"),
		XTAL_L("XCE1009"), XTAL_L("XCE1009:��`����Ă��Ȃ��ϐ� '%(name)s' �ɑ�����悤�Ƃ��܂��� �B"),
		XTAL_L("XCE1010"), XTAL_L("XCE1010:�s���Ȑ������e�����̃T�t�B�b�N�X�ł��B"),
		XTAL_L("XCE1011"), XTAL_L("XCE1011:�����񃊃e�����̓r���Ńt�@�C�����I���܂����B"),
		XTAL_L("XCE1012"), XTAL_L("XCE1012:�s���ȑ�����̍��ӂł��B"),
		XTAL_L("XCE1013"), XTAL_L("XCE1013:��r���Z���̌��ʂ����Z���悤�Ƃ��Ă��܂��B"),
		XTAL_L("XCE1014"), XTAL_L("XCE1014:�s���ȕ��������_�����e�����ł��B"),
		XTAL_L("XCE1015"), XTAL_L("XCE1015:�s����%(n)d�i���l���e�����̃T�t�B�b�N�X�ł��B"),
		XTAL_L("XCE1016"), XTAL_L("XCE1016:assert���̈����̐����s���ł��B"),
		XTAL_L("XCE1017"), XTAL_L("XCE1017:�s����%%�L�@���e�����ł��B"),
		XTAL_L("XCE1018"), XTAL_L("XCE1018:default�߂��d����`����܂����B"),
		XTAL_L("XCE1019"), XTAL_L("XCE1019:'%(name)s'�͑���s�\�ł��B"),
		XTAL_L("XCE1021"), XTAL_L("XCE1021:�R�����g�̓r���Ńt�@�C�����I���܂����B"),
		XTAL_L("XCE1022"), XTAL_L("XCE1022:�֐�����Ԃ��鑽�l�̍ő��255�ł��B"),
		XTAL_L("XCE1023"), XTAL_L("XCE1023:��`����Ă��Ȃ��C���X�^���X�ϐ��� '%(name)s' ���Q�Ƃ��Ă��܂��B"),
		XTAL_L("XCE1024"), XTAL_L("XCE1024:�����̃C���X�^���X�ϐ��� '%(name)s' �����ɒ�`����Ă��܂��B"),
		XTAL_L("XCE1025"), XTAL_L("XCE1025:��r���Z���̌��ʂ��Ĕ�r���悤�Ƃ��Ă��܂��B"),
		XTAL_L("XCE1026"), XTAL_L("XCE1026:�����X�R�[�v���ŁA�����ϐ��� '%(name)s' �����ɒ�`����Ă��܂��B"),
		XTAL_L("XCE1027"), XTAL_L("XCE1027:�R�[�h���傫�����āA�o�C�g�R�[�h�̐����Ɏ��s���܂����B"),
		XTAL_L("XCE1028"), XTAL_L("XCE1028:���Z�q�̑O��̋󔒂Ɖ��Z�q�̗D�揇�ʂ���v���Ă��܂���B�z�肵�Ă���D�揇�ʂƈقȂ��Ă���\��������܂��B"),
		XTAL_L("XCE1029"), XTAL_L("XCE1029:a&1�Ƃ����悤�ɁAbitwise and���������Ƃ��邱�Ƃ͈��S�̂��ߋ֎~����Ă��܂��B(a&1)!=0�Ƃ����`�Ŕ�r���Ă��������B"),
		XTAL_L("XCE1030"), XTAL_L("XCE1012:�s���Ȓ�`���̍��ӂł��B"),
		
		XTAL_L("XRE1001"), XTAL_L("XRE1001:'%(object)s' �֐��Ăяo���� '%(no)s'�Ԗڂ̈����̌^���s���ł��B'%(required)s'�^��v�����Ă��܂����A'%(type)s'�^�̒l���n����܂����B"),
		XTAL_L("XRE1002"), XTAL_L("XRE1002:�\�[�X�̃R���p�C�����A�R���p�C���G���[���������܂����B"),
		XTAL_L("XRE1003"), XTAL_L("XRE1003:�s���ȃC���X�^���X�ϐ��̎Q�Ƃł��B"),
		XTAL_L("XRE1004"), XTAL_L("XRE1004:�^�G���[�ł��B '%(required)s'�^��v�����Ă��܂����A'%(type)s'�^�̒l���n����܂����B"),
		XTAL_L("XRE1005"), XTAL_L("XRE1005:'%(object)s' �֐��Ăяo���̈����̐����s���ł��B'%(min)s'�ȏ�̈������󂯎��֐��ɁA%(value)s�̈�����n���܂����B"),
		XTAL_L("XRE1006"), XTAL_L("XRE1006:'%(object)s' �֐��Ăяo���̈����̐����s���ł��B'%(min)s'�ȏ�A'%(max)s'�ȉ��̈������󂯎��֐��ɁA'%(value)s'�̈�����n���܂����B"),
		XTAL_L("XRE1007"), XTAL_L("XRE1007:'%(object)s' �֐��Ăяo���̈����̐����s���ł��B���������Ȃ��֐��ɁA'%(value)s'�̈�����n���܂����B"),
		XTAL_L("XRE1008"), XTAL_L("XRE1008:'%(object)s'�̓V���A���C�Y�ł��܂���B"),
		XTAL_L("XRE1009"), XTAL_L("XRE1009:�s���ȃR���p�C���ς�Xtal�t�@�C���ł��B"),
		XTAL_L("XRE1010"), XTAL_L("XRE1010:�R���p�C���G���[���������܂����B"),
		XTAL_L("XRE1011"), XTAL_L("XRE1011:%(object)s :: '%(name)s' �͊��ɒ�`����Ă��܂��B"),
		XTAL_L("XRE1012"), XTAL_L("XRE1012:yield��fiber�̔���s���Ɏ��s����܂����B"),
		XTAL_L("XRE1013"), XTAL_L("XRE1013:%(object)s �ɃR���X�g���N�^���o�^����Ă��Ȃ����߁A�C���X�^���X�𐶐��ł��܂���B"),
		XTAL_L("XRE1014"), XTAL_L("XRE1014:�t�@�C�� '%(name)s' ���J���܂���B"),
		XTAL_L("XRE1015"), XTAL_L("XRE1015:%(object)s �͒�`����Ă��܂���B"),
		XTAL_L("XRE1016"), XTAL_L("XRE1016:�t�@�C�� '%(name)s' �̃R���p�C�����A�R���p�C���G���[���������܂����B"),
		XTAL_L("XRE1017"), XTAL_L("XRE1017:%(object)s :: %(primary_key)s # %(secondary_key)s�� %(accessibility)s �ł��B"),
		XTAL_L("XRE1018"), XTAL_L("XRE1018:���ɕ���ꂽ�X�g���[���ł��B"),
		XTAL_L("XRE1019"), XTAL_L("XRE1019:C++�Œ�`���ꂽ�N���X�̑��d�p���͏o���܂���B"),
		XTAL_L("XRE1020"), XTAL_L("XRE1020:�z��͈̔͊O�A�N�Z�X�ł��B"),
		XTAL_L("XRE1021"), XTAL_L("XRE1021:%(object)s �͒�`����Ă��܂���B'%(pick)s'�ƊԈႦ�Ă���\��������܂��B"),
		XTAL_L("XRE1023"), XTAL_L("XRE1023:1��蒷��������͔͈͉��Z�q�Ɏw��ł��܂���B"),
		XTAL_L("XRE1024"), XTAL_L("XRE1024:0���Z�G���[�ł��B"),
		XTAL_L("XRE1025"), XTAL_L("XRE1025:ChRange�͕�Ԃł���K�v������܂��B"),
		XTAL_L("XRE1026"), XTAL_L("XRE1026:xpeg�v�f�ɕϊ��ł��܂���B"),
		XTAL_L("XRE1027"), XTAL_L("XRE1027:cap�֐��̈������s���ł��Bcap(name: value)�Ƃ����悤�ɖ��O�t�������ɂ��邩�Acap(\"name\"), value)�Ƃ����悤�ɌĂ�ł��������B"),
		XTAL_L("XRE1028"), XTAL_L("XRE1028:final�}�[�N���t����ꂽ�N���X'%(name)s'���p�����悤�Ƃ��܂����B"),
		XTAL_L("XRE1029"), XTAL_L("XRE1029:C++�Œ�`���ꂽ�N���X'%(name)s'�́A�N���X�������̂݌p�����\�ł��B"),
		XTAL_L("XRE1030"), XTAL_L("XRE1030:�Öق̕ϐ��Q�Ƃ�����܂��B%(name)s"),
		XTAL_L("XRE1031"), XTAL_L("XRE1031:�V���O���g���N���X�̓V���O���g���N���X�łȂ��ƌp���ł��܂���B"),
		XTAL_L("XRE1032"), XTAL_L("XRE1032:�t�@�C��'%(name)s'���J���܂���B"),
		XTAL_L("XRE1033"), XTAL_L("XRE1033:�X�g���[���̏I�[�ȍ~��ǂݎ�낤�Ƃ��܂����B"),
		XTAL_L("XRE1034"), XTAL_L("XRE1034:�������[�v����������\��������xpeg�v�f�����s���悤�Ƃ��܂���"),
		XTAL_L("XRE1035"), XTAL_L("XRE1035:���s���Ŕ�yield���̃t�@�C�o�[�ɑ΂���s���ȑ���ł�"),	
		XTAL_L("XRE1036"), XTAL_L("XRE1036:'%(object)s' �֐��Ăяo���̈����̖��O���s���ł��B�֐����ŕK�v�Ƃ���Ă��Ȃ����O�t������'%(name)s'���n����܂���"),	
	};
	
	for(unsigned int i=0; i<sizeof(messages)/sizeof(*messages)/2; ++i){
		IDPtr key(*(LongLivedString*)messages[i*2+0]);
		text_map()->set_at(key, *(LongLivedString*)messages[i*2+1]);
	}
}
	
}
