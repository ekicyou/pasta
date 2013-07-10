using System;
using System.Collections.Generic;
using System.Text;

namespace System.Collections.Generic
{
    public partial class Deque<T>
    {
        private Lazy<DequeFrontQueue<T>> _FrontQueue;
        private Lazy<DequeBackQueue<T>> _BackQueue;

        /// <summary>
        /// 先頭から読み出すキューを取得します。
        /// </summary>
        public IQueue<T> FrontQueue { get { return _FrontQueue.Value; } }

        /// <summary>
        /// 後方から読み出すキューを取得します。
        /// </summary>
        public IQueue<T> BackQueue { get { return _BackQueue.Value; } }

        /// <summary>
        /// キュー初期化
        /// </summary>
        partial void InitQueue()
        {
            _FrontQueue = new Lazy<DequeFrontQueue<T>>(() => new DequeFrontQueue<T>(this));
            _BackQueue = new Lazy<DequeBackQueue<T>>(() => new DequeBackQueue<T>(this));
        }
    }


    internal abstract class DequeEmuQueueBase<T> : IQueue<T>
    {
        public abstract T Peek();
        public abstract T Dequeue();
        public abstract void Enqueue(T item);
        public abstract IEnumerator<T> GetEnumerator();
        public abstract T[] ToArray();

        #region 実装

        protected Deque<T> Deque { get; private set; }

        public DequeEmuQueueBase(Deque<T> deque)
        {
            Deque = deque;
        }

        public void Clear()
        {
            throw new NotImplementedException();
        }

        public bool Contains(T item)
        {
            return Deque.Contains(item);
        }

        IEnumerator IEnumerable.GetEnumerator()
        {
            return GetEnumerator();
        }

        public void CopyTo(Array array, int index)
        {
            throw new NotImplementedException();
        }

        public int Count { get { return Deque.Count; } }
        public bool IsSynchronized { get { return Deque.IsSynchronized; } }
        public object SyncRoot { get { return Deque.SyncRoot; } }

        #endregion
    }


    internal sealed class DequeFrontQueue<T> : DequeEmuQueueBase<T>
    {
        public DequeFrontQueue(Deque<T> deque) : base(deque) { }

        public override T Peek() { return Deque.PeekFront(); }
        public override T Dequeue() { return Deque.PopFront(); }
        public override void Enqueue(T item) { Deque.PushFront(item); }
        public override IEnumerator<T> GetEnumerator() { return Deque.GetFrontEnumerator(); }
        public override T[] ToArray() { return Deque.ToArray(); }
    }

    internal sealed class DequeBackQueue<T> : DequeEmuQueueBase<T>
    {
        public DequeBackQueue(Deque<T> deque) : base(deque) { }

        public override T Peek() { return Deque.PeekBack(); }
        public override T Dequeue() { return Deque.PopBack(); }
        public override void Enqueue(T item) { Deque.PushBack(item); }
        public override IEnumerator<T> GetEnumerator() { return Deque.GetBackEnumerator(); }
        public override T[] ToArray()
        {
            var list = new List<T>(this.Count);
            list.AddRange(this);
            return list.ToArray();
        }
    }

}
