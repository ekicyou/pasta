namespace NeoDatis.Odb.Core.Query.Criteria
{
	[System.Serializable]
	public abstract class AbstractExpression : NeoDatis.Odb.Core.Query.Criteria.IExpression
	{
		private NeoDatis.Odb.Core.Query.IQuery query;

		public AbstractExpression()
		{
		}

		/// <summary>Gets thes whole query</summary>
		/// <returns>The owner query</returns>
		public virtual NeoDatis.Odb.Core.Query.IQuery GetQuery()
		{
			return query;
		}

		public virtual void SetQuery(NeoDatis.Odb.Core.Query.IQuery query)
		{
			this.query = query;
		}

		public virtual bool CanUseIndex()
		{
			return false;
		}

		public abstract NeoDatis.Tool.Wrappers.List.IOdbList<string> GetAllInvolvedFields
			();

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			();

		public abstract bool Match(object arg1);

		public abstract void Ready();
	}
}
