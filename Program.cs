using System;
using System.IO;

namespace jcomp
{
    class Program
    {
        static void Main(string[] args)
        {
            using (var sw = new StreamWriter(File.OpenWrite(args[2])))
            {
                sw.WriteLine("v2.0 raw");
                foreach(string line in File.ReadAllLines(args[1]))
                {

                }
            }
        }
    }
}
