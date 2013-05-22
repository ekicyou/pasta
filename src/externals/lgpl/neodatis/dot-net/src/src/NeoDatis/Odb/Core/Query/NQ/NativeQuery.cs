using System;
namespace NeoDatis.Odb.Core.Query.NQ
{
	[System.Serializable]
	public abstract class NativeQuery : NeoDatis.Odb.Core.Query.AbstractQuery
	{
		public abstract bool Match(object @object);

		public abstract System.Type GetObjectType();

		public virtual System.Type[] GetObjectTypes()
		{
			System.Type[] classes = new System.Type[1];
			classes[0] = GetObjectType();
			return classes;
		}

		public virtual string[] GetIndexFields()
		{
			return new string[0];
		}

		public override void SetExecutionPlan(NeoDatis.Odb.Core.Query.Execution.IQueryExecutionPlan
			 plan)
		{
			executionPlan = plan;
		}
        public override void SetFullClassName(Type type)
        {
            //fullClassName = OdbClassUtil.GetFullName(type);
        }
	}
}
