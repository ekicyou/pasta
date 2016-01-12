
namespace System.Collections.Generic
{
    public partial class Deque<T>
    {
        #region Enumerator Class

        [Serializable()]
        private abstract class EnumeratorBase : IEnumerator<T>
        {
            protected Deque<T> owner;
            protected Node currentNode;

            protected abstract Node StartNode { get; }
            protected abstract Node NextNode { get; }

            private T current = default(T);

            private bool moveResult = false;

            private long version;

            // A value indicating whether the enumerator has been disposed.
            private bool disposed = false;

            public EnumeratorBase(Deque<T> owner)
            {
                this.owner = owner;
                this.version = owner.version;
                currentNode = StartNode;
            }

            #region IEnumerator Members

            public void Reset()
            {
                #region Require

                if (disposed) {
                    throw new ObjectDisposedException(this.GetType().Name);
                }
                else if (version != owner.version) {
                    throw new InvalidOperationException(
                        "The Deque was modified after the enumerator was created.");
                }

                #endregion

                currentNode = StartNode;
                moveResult = false;
            }

            public object Current
            {
                get
                {
                    #region Require

                    if (disposed) {
                        throw new ObjectDisposedException(this.GetType().Name);
                    }
                    else if (!moveResult) {
                        throw new InvalidOperationException(
                            "The enumerator is positioned before the first " +
                            "element of the Deque or after the last element.");
                    }

                    #endregion

                    return current;
                }
            }

            public bool MoveNext()
            {
                #region Require

                if (disposed) {
                    throw new ObjectDisposedException(this.GetType().Name);
                }
                else if (version != owner.version) {
                    throw new InvalidOperationException(
                        "The Deque was modified after the enumerator was created.");
                }

                #endregion

                if (currentNode != null) {
                    current = currentNode.Value;
                    currentNode = currentNode.Next;

                    moveResult = true;
                }
                else {
                    moveResult = false;
                }

                return moveResult;
            }

            #endregion

            #region IEnumerator<T> Members

            T IEnumerator<T>.Current
            {
                get
                {
                    #region Require

                    if (disposed) {
                        throw new ObjectDisposedException(this.GetType().Name);
                    }
                    else if (!moveResult) {
                        throw new InvalidOperationException(
                            "The enumerator is positioned before the first " +
                            "element of the Deque or after the last element.");
                    }

                    #endregion

                    return current;
                }
            }

            #endregion

            #region IDisposable Members

            public void Dispose()
            {
                disposed = true;
            }

            #endregion
        }


        [Serializable()]
        private sealed class FrontEnumerator : EnumeratorBase
        {
            protected override Node StartNode { get { return owner.front; } }
            protected override Node NextNode { get { return currentNode.Next; } }
            public FrontEnumerator(Deque<T> owner) : base(owner) { }
        }

        [Serializable()]
        private sealed class BackEnumerator : EnumeratorBase
        {
            protected override Node StartNode { get { return owner.back; } }
            protected override Node NextNode { get { return currentNode.Previous; } }
            public BackEnumerator(Deque<T> owner) : base(owner) { }
        }

        #endregion
    }
}