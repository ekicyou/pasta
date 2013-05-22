using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace Pasta.Util.Disposables
{
    /// <summary>
    /// Represents an Action-based disposable.
    /// </summary>
    internal sealed class AnonymousDisposable : IDisposable
    {
        private readonly Action dispose;
        private int isDisposed;
        /// <summary>
        /// Constructs a new disposable with the given action used for disposal.
        /// </summary>
        /// <param name="dispose">Disposal action.</param>
        public AnonymousDisposable(Action dispose)
        {
            this.dispose = dispose;
        }
        /// <summary>
        /// Calls the disposal action.
        /// </summary>
        public void Dispose()
        {
            if (Interlocked.Exchange(ref this.isDisposed, 1) == 0)
            {
                this.dispose();
            }
        }
    }
}
