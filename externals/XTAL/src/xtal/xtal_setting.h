/** \file src/xtal/xtal_setting.h
* \brief src/xtal/xtal_setting.h
*/

#ifndef XTAL_SETTING_H_INCLUDE_GUARD
#define XTAL_SETTING_H_INCLUDE_GUARD

#pragma once

/**
* \brief パーサ使用off
*/
//#define XTAL_NO_PARSER

/**
* \brief wchar_t使用on
*/
//#define XTAL_USE_WCHAR

/**
* \brief スレッド使用off
*/
#define XTAL_NO_THREAD

/**
* \brief Xtalスレッドモデル2を使用する
* 同じEnvironmentに所属するVMachine同士でも並列に実行することを可能にするが、
* 細かい排他制御が入るため並列に実行しない場合遅くなる。
*/
//#define XTAL_USE_THREAD_MODEL2

/**
* \brief pthreadのスレッドローカルストレージを使う
*/
//#define XTAL_USE_PTHREAD_TLS

/**
* \brief 小さいサイズのメモリ確保にXtal独自のアロケータを使わない
*/
//#define XTAL_NO_SMALL_ALLOCATOR


//#define XTAL_CHECK_REF_COUNT

#endif // XTAL_SETTING_H_INCLUDE_GUARD
