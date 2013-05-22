namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	[System.Serializable]
	public abstract class CustomQueryFieldAction : NeoDatis.Odb.Core.Query.Values.AbstractQueryFieldAction
		, NeoDatis.Odb.Core.Query.Values.ICustomQueryFieldAction
	{
		public CustomQueryFieldAction() : base(null, null, true)
		{
		}

		public virtual void SetAlias(string alias)
		{
			this.alias = alias;
		}

		public virtual void SetAttributeName(string attributeName)
		{
			this.attributeName = attributeName;
		}

		public override NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction Copy()
		{
			try
			{
				NeoDatis.Odb.Core.Query.Values.ICustomQueryFieldAction cqfa = (NeoDatis.Odb.Core.Query.Values.ICustomQueryFieldAction
					)System.Activator.CreateInstance(GetType());
				cqfa.SetAttributeName(attributeName);
				cqfa.SetAlias(alias);
				return cqfa;
			}
			catch (System.Exception)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ValuesQueryErrorWhileCloningCustumQfa
					.AddParameter(GetType().FullName));
			}
		}

        public abstract override void End();

        public abstract override object GetValue();

        public abstract override void Start();
	}
}
