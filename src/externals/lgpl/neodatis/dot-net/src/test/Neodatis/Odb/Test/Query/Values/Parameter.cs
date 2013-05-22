using NUnit.Framework;
namespace NeoDatis.Odb.Test.Query.Values
{
	/// <author>olivier</author>
	public class Parameter
	{
		private string name;

		private object value;

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetName(string name)
		{
			this.name = name;
		}

		public virtual object GetValue()
		{
			return value;
		}

		public virtual void SetValue(object value)
		{
			this.value = value;
		}

		public Parameter(string name, object value) : base()
		{
			this.name = name;
			this.value = value;
		}
	}
}
