using System;
using System.Collections.Generic;
using System.Text;
using System.IO;

namespace NeoDatis.Tool.Wrappers.IO{
  public class OdbFile
    {
        private string fileName;


        public OdbFile(String fileName)
        {
            this.fileName = fileName;
            // Converts path (with directory info)
            this.fileName = GetFullPath();
        }

        public String GetDirectory()
        {
            return System.IO.Path.GetDirectoryName(fileName);
        }

        public String GetCleanFileName()
        {
            return System.IO.Path.GetFileName(fileName);
        }

        public String GetFullPath()
        {
            return System.IO.Path.GetFullPath(fileName);
        }

        public bool Exists()
        {
            return System.IO.File.Exists(fileName);
        }

        public void Clear()
        {
            
        }

        public OdbFile GetParentFile()
        {
            DirectoryInfo di = new DirectoryInfo(GetDirectory());
            return new OdbFile(di.Parent.FullName);
        }

        public void Mkdirs()
        {
            DirectoryInfo di = new DirectoryInfo(GetDirectory());
            //TODO check if it creates all sub directories
            di.Create();
        }

        public bool Delete()
        {
            bool fileExists = Exists();
            if (!fileExists)
            {
                return false;
            }
            System.IO.File.Delete(fileName);
            return !Exists();
        }

        public void Create()
        {
            System.IO.File.Create(fileName);
        }

        public void Release()
        {

        }

    }
}