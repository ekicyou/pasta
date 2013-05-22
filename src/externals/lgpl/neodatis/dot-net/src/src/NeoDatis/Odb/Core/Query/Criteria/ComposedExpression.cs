using NeoDatis.Tool.Wrappers.List;
using System.Collections;
namespace NeoDatis.Odb.Core.Query.Criteria
{
	[System.Serializable]
	public abstract class ComposedExpression : NeoDatis.Odb.Core.Query.Criteria.AbstractExpression
	{
		protected NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Query.Criteria.ICriterion
			> criteria;

		public ComposedExpression()
		{
			criteria = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Query.Criteria.ICriterion
				>(5);
		}

		public virtual NeoDatis.Odb.Core.Query.Criteria.ComposedExpression Add(NeoDatis.Odb.Core.Query.Criteria.ICriterion
			 criterion)
		{
			criteria.Add(criterion);
			return this;
		}

		public override IOdbList<string> GetAllInvolvedFields
			()
		{
			IEnumerator iterator = criteria.GetEnumerator();
			ICriterion criterion = null;
			IOdbList<string> fields = new OdbArrayList<string>(10);
			while (iterator.MoveNext())
			{
				criterion = (ICriterion)iterator.Current;
                IOdbList<string> l = criterion.GetAllInvolvedFields();
                // check duplicate
                for (int i = 0; i < l.Count; i++)
                {
                    string f = l.Get(i);
                    if(!fields.Contains(f))
                    {
                        fields.Add(f);
                    }
                }
			}
			return fields;
		}

		public virtual bool IsEmpty()
		{
			return criteria.IsEmpty();
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap map = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
				();
			System.Collections.IEnumerator iterator = criteria.GetEnumerator();
			NeoDatis.Odb.Core.Query.Criteria.ICriterion criterion = null;
			while (iterator.MoveNext())
			{
				criterion = (NeoDatis.Odb.Core.Query.Criteria.ICriterion)iterator.Current;
				map.PutAll(criterion.GetValues());
			}
			return map;
		}

		public virtual int GetNbCriteria()
		{
			return criteria.Count;
		}

		public virtual NeoDatis.Odb.Core.Query.Criteria.ICriterion GetCriterion(int index
			)
		{
			return criteria[index];
		}

		public override void Ready()
		{
		}
	}
}
