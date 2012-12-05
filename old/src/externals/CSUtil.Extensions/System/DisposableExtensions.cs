
namespace System
{
    public static class DisposableExtensions
    {
        public static void DisposeNotNull(this IDisposable dis)
        {
            if (dis == null) return;
            dis.Dispose();
        }
    }
}