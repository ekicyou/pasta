using System;
using System.Collections.Generic;
using System.Text;
using System.Collections;

namespace NeoDatis.Btree
{
    abstract public class IterarorAdapter:IEnumerator 
    {
        public object Current { 
            get
            {
                return GetCurrent();
            }
        }
        public abstract object GetCurrent();
        public abstract bool MoveNext();
        public abstract void Reset();

        
    }
}
