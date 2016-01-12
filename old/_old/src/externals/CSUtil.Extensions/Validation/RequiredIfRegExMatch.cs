using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    /// <summary>
    /// 正規表現がマッチした場合に必須であることを示します。
    /// </summary>
    public class RequiredIfRegExMatchAttribute : RequiredIfAttribute
    {
        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="dependentProperty"></param>
        /// <param name="pattern"></param>
        public RequiredIfRegExMatchAttribute(string dependentProperty, string pattern) : base(dependentProperty, Operator.RegExMatch, pattern) { }
    }
}
