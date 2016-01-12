using System.ComponentModel;
using System.Linq.Expressions;
using System.Reactive.Linq;
using System.Reflection;

namespace System.Windows
{
    public static class DepedencyPropertyRx
    {
        /// <summary>
        /// 依存関係プロパティの値変更イベントをキャプチャします。
        /// </summary>
        /// <param name="src"></param>
        /// <param name="dp"></param>
        /// <returns></returns>
        public static IObservable<EventArgs> RxValueChanged(
            this DependencyObject src, DependencyProperty dp)
        {          
            var srcType = src.GetType();
            var desc = DependencyPropertyDescriptor.FromProperty(dp, srcType);
            return Observable
                .FromEventPattern(
                    add => { desc.AddValueChanged(src, add); },
                    del => { desc.RemoveValueChanged(src, del); })
                .Select(a => a.EventArgs)
                ;
        }

        /// <summary>
        /// 依存関係プロパティの値変更をキャプチャし、その値の通知を作成します。
        /// </summary>
        /// <typeparam name="TSrc"></typeparam>
        /// <typeparam name="TValue"></typeparam>
        /// <param name="src"></param>
        /// <param name="property"></param>
        /// <returns></returns>
        public static IObservable<TValue> RxFromDPValue<TSrc, TValue>(
            this TSrc src, Expression<Func<TSrc, TValue>> property)
            where TSrc : DependencyObject
        {
            var prop_name =property.ToPropertyName();
            var fi = typeof(TSrc).GetField(prop_name + "Property", BindingFlags.Public | BindingFlags.Static | BindingFlags.FlattenHierarchy);
            var dp = fi.GetValue(src) as DependencyProperty;
            var func = property.Compile();
            return src
                .RxValueChanged(dp)
                .Select(a => func(src));
        }

        /// <summary>
        /// 依存関係プロパティの値変更をキャプチャし、その値の通知を作成します。
        /// 最初に現在の値を返します。
        /// </summary>
        /// <typeparam name="TSrc"></typeparam>
        /// <typeparam name="TValue"></typeparam>
        /// <param name="src"></param>
        /// <param name="property"></param>
        /// <returns></returns>
        public static IObservable<TValue> RxFromDPValueEx<TSrc, TValue>(
            this TSrc src, Expression<Func<TSrc, TValue>> property)
            where TSrc : DependencyObject
        {
            var func = property.Compile();
            return Observable
                .Return(func(src))
                .Concat(src.RxFromDPValue(property));
        }



    }
}
