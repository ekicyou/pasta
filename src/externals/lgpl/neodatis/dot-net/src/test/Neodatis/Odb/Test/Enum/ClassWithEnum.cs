using System;
using System.Collections.Generic;
using System.Text;

namespace Test.Neodatis.Odb.Test.Enum
{
    public enum ObjectType
    {
        Small,
        Medium,
        Big
    }
    class ClassWithEnum
    {
        protected string name;
        ObjectType type;

        public ClassWithEnum(string name, ObjectType type)
        {
            this.name = name;
            this.type = type;
        }

        public string GetName()
        {
            return name;
        }
        public ObjectType GetObjectType()
        {
            return type;
        }


        internal void SetObjectType(ObjectType objectType)
        {
            this.type = objectType;
        }
    }
}
