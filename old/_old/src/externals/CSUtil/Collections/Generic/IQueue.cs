using System;
using System.Collections;
using System.Collections.Generic;
using System.Text;

namespace System.Collections.Generic
{
    /// <summary>
    /// Queueのインターフェースです。
    /// </summary>
    /// <typeparam name="T"></typeparam>
    public interface IQueue<T> : IEnumerable<T>, ICollection, IEnumerable
    {
        /// <summary>
        /// キュー からすべてのオブジェクトを削除します。
        /// </summary>
        void Clear();

        /// <summary>
        /// ある要素が キュー  内に存在するかどうかを判断します。
        /// </summary>
        /// <param name="item">キュー  内で検索するオブジェクト。参照型の場合、null の値を使用できます。</param>
        /// <returns>item が キュー  に存在する場合は true。それ以外の場合は false。</returns>
        bool Contains(T item);

        /// <summary>
        /// キュー  の先頭にあるオブジェクトを削除し、返します。
        /// </summary>
        /// <returns>キュー  の先頭から削除されたオブジェクト。</returns>
        /// <exception cref="System.InvalidOperationException">キュー  が空です。</exception>
        T Dequeue();

        /// <summary>
        /// キュー  の末尾にオブジェクトを追加します。
        /// </summary>
        /// <param name="item">キュー  に追加するオブジェクト。参照型の場合、null の値を使用できます。</param>
        void Enqueue(T item);

        /// <summary>
        /// キュー  の先頭にあるオブジェクトを削除せずに返します。
        /// </summary>
        /// <returns>キュー  の先頭にあるオブジェクト。</returns>
        /// <exception cref="System.InvalidOperationException">キュー  が空です。</exception>
        T Peek();

        /// <summary>
        /// キュー  の要素を新しい配列にコピーします。
        /// </summary>
        /// <returns>キュー  からコピーした要素を格納する新しい配列。</returns>
        T[] ToArray();

    }
}