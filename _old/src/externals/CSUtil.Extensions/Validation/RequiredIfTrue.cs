using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    /// <summary>
    /// 指定したプロパティが真である場合に必須であることを示します。
    /// </summary>
    public class RequiredIfTrueAttribute : RequiredIfAttribute
    {
        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="dependentProperty"></param>
        public RequiredIfTrueAttribute(string dependentProperty) : base(dependentProperty, Operator.EqualTo, true) { }
    }
}
