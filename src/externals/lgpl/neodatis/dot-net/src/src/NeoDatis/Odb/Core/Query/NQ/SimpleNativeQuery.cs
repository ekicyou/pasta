using System;
namespace NeoDatis.Odb.Core.Query.NQ
{
	[System.Serializable]
	public class SimpleNativeQuery : NeoDatis.Odb.Core.Query.AbstractQuery
	{
        public override void SetFullClassName(Type type)
        {
            // nothing
        }
	}
}
