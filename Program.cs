using System;
using System.Globalization;
using System.IO;

namespace jcomp
{
    class Program
    {
        enum OpCode {
            Load,
            Store,
            And,
            Or,
            Add,
            Not,
            Copy,
            Xor,
            LoadLo,
            LoadLo9,
            LoadLoA,
            LoadLoB,
            LoadHi,
            LoadHiD,
            LoadHiE,
            LoadHiF
        }

        enum Reg {
            A,
            B,
            C,
            PC
        }

        static void Main(string[] args)
        {
            Console.WriteLine("v2.0 raw");

            string line;
            while (null != (line = Console.ReadLine()))
            {
                string[] tokens = line.Split(" ", StringSplitOptions.RemoveEmptyEntries);
                OpCode op = Enum.Parse<OpCode>(tokens[0], ignoreCase: true);
                string arg0 = tokens[1];
                Reg reg1 = Enum.Parse<Reg>(tokens[2], ignoreCase: true);

                switch(op)
                {
                    case OpCode.Load:
                    case OpCode.Store:
                        if (arg0[0] != '[' || arg0[2] != ']')
                        {
                            throw new ArgumentException();
                        }
                        arg0 = arg0[1].ToString();
                        break;
                }

                int instruction = ((int)op) << 4;
                switch(op)
                {
                    case OpCode.Load:
                    case OpCode.Store:
                    case OpCode.And:
                    case OpCode.Or:
                    case OpCode.Add:
                    case OpCode.Not:
                    case OpCode.Copy:
                    case OpCode.Xor:
                        Reg reg0 = Enum.Parse<Reg>(arg0, ignoreCase: true);
                        instruction |= (int)reg1;
                        instruction |= ((int)reg0) << 2;
                        break;

                    case OpCode.LoadLo:
                    case OpCode.LoadLo9:
                    case OpCode.LoadLoA:
                    case OpCode.LoadLoB:
                    case OpCode.LoadHi:
                    case OpCode.LoadHiD:
                    case OpCode.LoadHiE:
                    case OpCode.LoadHiF:
                        int imm = int.Parse(arg0, NumberStyles.HexNumber);
                        instruction |= (int)reg1;
                        instruction |= (imm << 2);
                        break;
                    default:
                        throw new NotImplementedException(op.ToString());
                }

                Console.Write("{0:x02} ", (byte)instruction);
            }
            Console.WriteLine();
        }
    }
}
