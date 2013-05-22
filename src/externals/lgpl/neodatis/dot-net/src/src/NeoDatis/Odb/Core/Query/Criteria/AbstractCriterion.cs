namespace NeoDatis.Odb.Core.Query.Criteria
{
	/// <summary>An adapter for Criterion.</summary>
	/// <remarks>An adapter for Criterion.</remarks>
	/// <author>olivier s</author>
	[System.Serializable]
	public abstract class AbstractCriterion : NeoDatis.Odb.Core.Query.Criteria.ICriterion
	{
		public virtual bool CanUseIndex()
		{
			return false;
		}

		/// <summary>The query containing the criterion</summary>
		private NeoDatis.Odb.Core.Query.IQuery query;

		/// <summary>The name of the attribute involved by this criterion</summary>
		protected string attributeName;

		public AbstractCriterion(string fieldName)
		{
			this.attributeName = fieldName;
		}

		public virtual bool Match(NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
			 aoi)
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
				)aoi;
			object aoiValue = nnoi.GetValueOf(attributeName);
			return Match(aoiValue);
		}

		public virtual bool Match(NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
			 attributeValues)
		{
			return Match(attributeValues.GetAttributeValue(attributeName));
		}

		public abstract bool Match(object valueToMatch);

		public virtual NeoDatis.Odb.Core.Query.Criteria.IExpression And(NeoDatis.Odb.Core.Query.Criteria.ICriterion
			 criterion)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.And().Add(this).Add(criterion);
		}

		public virtual NeoDatis.Odb.Core.Query.Criteria.IExpression Or(NeoDatis.Odb.Core.Query.Criteria.ICriterion
			 criterion)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.Or().Add(this).Add(criterion);
		}

		public virtual NeoDatis.Odb.Core.Query.Criteria.IExpression Not()
		{
			return new NeoDatis.Odb.Core.Query.Criteria.Not(this);
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

		/// <returns>The attribute involved in the criterion</returns>
		public virtual string GetAttributeName()
		{
			return attributeName;
		}

		/// <summary>An abstract criterion only restrict one field =&gt; it returns a list of one field!
		/// 	</summary>
		/// <returns>The list of involved field of the criteria</returns>
		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<string> GetAllInvolvedFields(
			)
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<string> l = new NeoDatis.Tool.Wrappers.List.OdbArrayList
				<string>(1);
			l.Add(attributeName);
			return l;
		}

		public virtual void SetAttributeName(string attributeName)
		{
			this.attributeName = attributeName;
		}

		public abstract NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			();

		public abstract void Ready();
	}
}
