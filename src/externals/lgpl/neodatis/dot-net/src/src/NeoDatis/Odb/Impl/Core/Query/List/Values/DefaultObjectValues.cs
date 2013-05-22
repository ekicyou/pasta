namespace NeoDatis.Odb.Impl.Core.Query.List.Values
{
	/// <author>osmadja</author>
	[System.Serializable]
	public class DefaultObjectValues : NeoDatis.Odb.ObjectValues
	{
		private object[] valuesByIndex;

		/// <summary>key=alias,value=value</summary>
		private NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, object> valuesByAlias;

		public DefaultObjectValues(int size)
		{
			valuesByIndex = new object[size];
			valuesByAlias = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, object>();
		}

		public virtual void Set(int index, string alias, object value)
		{
			valuesByIndex[index] = value;
			valuesByAlias.Add(alias, value);
		}

		public virtual object GetByAlias(string alias)
		{
			object o = valuesByAlias[alias];
			if (o == null && !valuesByAlias.ContainsKey(alias))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ValuesQueryAliasDoesNotExist
					.AddParameter(alias).AddParameter(valuesByAlias.Keys));
			}
			return o;
		}

		public virtual object GetByIndex(int index)
		{
			return valuesByIndex[index];
		}

		public virtual object[] GetValues()
		{
			return valuesByIndex;
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			System.Collections.Generic.IEnumerator<string> aliases = valuesByAlias.Keys.GetEnumerator
				();
			string alias = null;
			object @object = null;
			while (aliases.MoveNext())
			{
				alias = aliases.Current;
				@object = valuesByAlias[alias];
				buffer.Append(alias).Append("=").Append(@object).Append(",");
			}
			return buffer.ToString();
		}
	}
}
