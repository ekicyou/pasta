using NeoDatis.Tool.Wrappers;
using System;
using NeoDatis.Odb.Core.Query;
using NeoDatis.Odb.Core.Query.Criteria;
using NeoDatis.Tool.Wrappers.List;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core.Query.Execution;
namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	[System.Serializable]
	public class CriteriaQuery : AbstractQuery
	{
		private string fullClassName;

		private ICriterion criterion;

		public virtual bool HasCriteria()
		{
			return criterion != null;
		}

		public virtual bool Match(AbstractObjectInfo
			 aoi)
		{
			if (criterion == null)
			{
				return true;
			}
			return criterion.Match(aoi);
		}

		public virtual bool Match(System.Collections.IDictionary map)
		{
			if (criterion == null)
			{
				return true;
			}
			return criterion.Match(map);
		}

		public CriteriaQuery(System.Type aClass, NeoDatis.Odb.Core.Query.Criteria.ICriterion
			 criteria) : this(OdbClassUtil.GetFullName(aClass), criteria)
		{
		}

        public CriteriaQuery(System.Type aClass)
            : this(OdbClassUtil.GetFullName(aClass))
		{
		}

        public CriteriaQuery(ICriterion  criteria)
        {
            if (criteria != null)
            {
                this.criterion = criteria;
                this.criterion.SetQuery(this);
            }
        }

        public CriteriaQuery()
            
        {
        }

        public override void SetFullClassName(Type type)
        {
            fullClassName = OdbClassUtil.GetFullName(type);
        }

        

		public CriteriaQuery(string aFullClassName)
		{
			this.fullClassName = aFullClassName;
			this.criterion = null;
		}

		public CriteriaQuery(string aFullClassName, ICriterion criteria)
		{
			this.fullClassName = aFullClassName;
			if (criteria != null)
			{
				this.criterion = criteria;
				this.criterion.SetQuery(this);
			}
		}

		public virtual string GetFullClassName()
		{
			return fullClassName;
		}

		public virtual ICriterion GetCriteria()
		{
			return criterion;
		}

		public override string ToString()
		{
			if (criterion == null)
			{
				return "no criterion";
			}
			return criterion.ToString();
		}

		public virtual IOdbList<string> GetAllInvolvedFields(
			)
		{
			if (criterion == null)
			{
				return new NeoDatis.Tool.Wrappers.List.OdbArrayList<string>();
			}
			return criterion.GetAllInvolvedFields();
		}

		public virtual void SetCriterion(ICriterion criterion
			)
		{
			this.criterion = criterion;
		}

		public override void SetExecutionPlan(IQueryExecutionPlan plan)
		{
			executionPlan = plan;
		}
	}
}
