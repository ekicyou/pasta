using System;
using System.ComponentModel;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Linq.Expressions;
using System.Windows.Threading;
using System.Text;

namespace CSUtil.ComponentModel
{
    public class RxValidationProperty<T> : RxProperty<T>, IDataErrorInfo
    {

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        /// <param name="value"></param>
        public RxValidationProperty(Dispatcher dispatcher, T value) : base(dispatcher, value) { Init(); }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public RxValidationProperty(Dispatcher dispatcher) : base(dispatcher) { Init(); }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public RxValidationProperty(T value) : base(value) { Init(); }

        /// <summary>
        /// コンストラクタ。
        /// </summary>
        public RxValidationProperty() : base() { Init(); }


        private void Init()
        {
            HasValidate = true;
            Tasks.Add(this.Subscribe(a => HasValidate = true));
        }


        #region IDataErrorInfo メンバー

        public string this[string columnName]
        {
            get
            {
                if ("Value" != columnName) return null;
                return Error;
            }
        }
        public string Error
        {
            get
            {
                if (!HasValidate) return ErrorText;
                HasValidate = false;
                ErrorText = Validate();
                return ErrorText;
            }
        }



        /// <summary>検証が必要なときはtrue</summary>
        private bool HasValidate { get; set; }

        /// <summary>現在のエラーテキスト</summary>
        private string ErrorText { get; set; }


        /// <summary>ValidationContext</summary>
        ValidationContext ValidationContext;

        /// <summary>ValidationAttributeの一覧</summary>
        private ValidationAttribute[] Attributes { get; set; }


        /// <summary>ValidateErrorFuncのチェック関数</summary>
        private Func<T, string, string> ValidateErrorFunc { get; set; }

        #endregion
        #region 検証処理登録関数
        /// <summary>
        /// プロパティのValidationAttributeを抽出して取り込みます。
        /// </summary>
        /// <param name="model"></param>
        /// <param name="property"></param>
        /// <returns></returns>
        public RxValidationProperty<T> SetValidateAttribute<TModel>(TModel model, Expression<Func<TModel, RxValidationProperty<T>>> property)
        {
            var member = property.GetPropertyMember();
            var allAttrs = member.GetCustomAttributes(true);
            Attributes = allAttrs.OfType<ValidationAttribute>().ToArray();

            ValidationContext = new ValidationContext(model, null, null) { MemberName = member.Name };
            var dispAttr = allAttrs.OfType<DisplayAttribute>().FirstOrDefault();
            if (dispAttr != null) ValidationContext.DisplayName = dispAttr.Name;
            else ValidationContext.DisplayName = member.Name;
            return this;
        }

        /// <summary>
        /// ValidateErrorFuncを登録します。
        /// </summary>
        /// <param name="validate"></param>
        /// <returns></returns>
        public RxValidationProperty<T> AddValidateError(Func<T, string> validate)
        {
            if (validate == null) throw new ArgumentNullException("validate");
            if (ValidateErrorFunc == null) {
                ValidateErrorFunc = (check, t1) => JoinErrorText(t1, validate(check));
            }
            else {
                var f1 = ValidateErrorFunc;
                ValidateErrorFunc = (check, t1) => JoinErrorText(f1(check, t1), validate(check));
            }
            return this;
        }
        private static string JoinErrorText(string t1, string t2)
        {
            if (string.IsNullOrWhiteSpace(t2)) return t1;
            if (string.IsNullOrWhiteSpace(t1)) return t2;
            return t1 + "\r\n" + t2;
        }


        #endregion
        #region 検証関数

        /// <summary>
        /// 検証結果を返します。
        /// </summary>
        /// <returns></returns>
        public string Validate()
        {
            var text = ValidateAttributes();
            if (ValidateErrorFunc != null) text = ValidateErrorFunc(Value, text);
            text = text.Trim();
            if (string.IsNullOrWhiteSpace(text)) return null;
            return text;
        }

        /// <summary>
        /// ValidateAttributeの検証処理。
        /// </summary>
        /// <returns></returns>
        private string ValidateAttributes()
        {
            if (ValidationContext == null) return "";
            if (Attributes == null) return "";
            var text = "";
            foreach (var item in Attributes) {
                try {
                    item.Validate(Value, ValidationContext);
                }
                catch (ValidationException valErr) {
                    text = JoinErrorText(text, valErr.Message);
                }
            }
            return text;
        }



        #endregion

    }
}
