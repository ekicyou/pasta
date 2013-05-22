namespace NeoDatis.Btree.Impl
{
	[System.Serializable]
	public class KeyAndValue : NeoDatis.Btree.IKeyAndValue
	{
		private System.IComparable key;

		private object value;

		public KeyAndValue(System.IComparable key, object value)
		{
			this.key = key;
			this.value = value;
		}

		public override string ToString()
		{
			return new System.Text.StringBuilder("(").Append(key).Append("=").Append(value).Append
				(") ").ToString();
		}

		public virtual System.IComparable GetKey()
		{
			return key;
		}

		public virtual void SetKey(System.IComparable key)
		{
			this.key = key;
		}

		public virtual object GetValue()
		{
			return value;
		}

		public virtual void SetValue(object value)
		{
			this.value = value;
		}
	}
}
