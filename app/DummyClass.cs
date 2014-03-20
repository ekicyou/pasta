using System;
using System.Collections.Generic;
using System.Linq;
using System.Web;
using System.Diagnostics;
using CSUtil.Reflection;

namespace pasta
{
    public class DummyClass
    {
        public void DummyMethod()
        {
            Debug.WriteLine(AssemblyUtil.GetCallingAssemblyPath());
        }
    }
}