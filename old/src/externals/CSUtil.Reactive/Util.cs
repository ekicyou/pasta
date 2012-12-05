using System;
using System.Reflection;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Linq.Expressions;

namespace System
{
    internal static class Util
    {
        /// <summary>
        /// 'x => x.SomeProperty'形式の式ツリーからプロパティ名を取得します。
        /// </summary>
        /// <typeparam name="TObj"></typeparam>
        /// <typeparam name="TRet"></typeparam>
        /// <param name="Property"></param>
        /// <returns></returns>
        internal static string ToPropertyName<TObj, TRet>(this Expression<Func<TObj, TRet>> Property)
        {
            return Property.GetPropertyMember().Name;
        }

        /// <summary>
        /// 'x => x.SomeProperty'形式の式ツリーからプロパティMemberInfoを取得します。
        /// </summary>
        /// <typeparam name="TObj"></typeparam>
        /// <typeparam name="TRet"></typeparam>
        /// <param name="Property"></param>
        /// <returns></returns>
        internal static MemberInfo GetPropertyMember<TObj, TRet>(this Expression<Func<TObj, TRet>> Property)
        {
            try {
                var prop_expr = Property.Body as MemberExpression;
                if (prop_expr.Expression.NodeType != ExpressionType.Parameter) {
                    throw new ArgumentException("Property expression must be of the form 'x => x.SomeProperty'");
                }
                var member = prop_expr.Member;
                return member;
            }
            catch (NullReferenceException) {
                throw new ArgumentException("Property expression must be of the form 'x => x.SomeProperty'");
            }

        }



    }
}
