using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using NDde.Server;
using System.Threading;
using System.IO;

namespace new_robot_uralpro
{
    //DDE сервер для Квика
    class ClassDdeServer
    {
       
        public ClassDdeServer()
        {
        }

        //Запуск сервера
        public void mainDdeServer()
        {
            DdeServer server = new MyServer("myapp");//myapp - название сервиса (указать в Квике)
            server.Register();
            Thread.Sleep(Timeout.Infinite);
            GC.KeepAlive(server);
        }
        public class MyServer : DdeServer
        {
            public MyServer(string service)
                : base(service)
            {
            }
            TimeSpan time = new TimeSpan(0, 0, 0);
            string name = "";
            string operation = "";
            double price = 0;
            int num = 0;
            int quantity = 0;
            int execode = 0;
            int ordercode = 0;
            double vol = 0;
            int comment = -1;
            Encoding encoding = Encoding.GetEncoding(1251);

            //Приход данных с сервера
            protected override PokeResult OnPoke(DdeConversation conversation, string item, byte[] data, int format)
            {
               
                int rows=0, colu=0;
                byte[] ch=new byte[0];
                byte nstr2=new byte();

                MemoryStream ms = new MemoryStream(data);
                BinaryReader br = new BinaryReader(ms);

                br.ReadUInt32();
                //br.ReadUInt16();
                rows = br.ReadUInt16();
                colu = br.ReadUInt16();


                switch (conversation.Topic)
                {
                    //Данные синтетического индекса
                        /* Настройки таблиц вывода по DDE в Квике:
                         * Рабочая книга - rec
                         * Лист - sintind
                         * Столбцы должны располагаться в следующем порядке (строго!):
                         * Номер
                         * Код Бумаги
                         * Операция
                         * Цена
                         * Кол-во
                         * Время
                         * Других столбцов быть не должно!
                         * Снять все галочки, кроме Вывод при нажатии Ctrl+Shift+L
                         */
                    case"[rec]sintind":
                        for (int i = 1; i <= rows; i++)
                        {
                            br.ReadUInt16();
                            br.ReadUInt16();
                            num =(int) br.ReadDouble();

                            br.ReadUInt32();
                            name = getString(br);

                            operation = getString(br);

                            br.ReadUInt32();
                            price=br.ReadDouble();
                            vol=br.ReadDouble();

                            br.ReadUInt32();
                            string pr = getString(br);
                            time = Convert.ToDateTime(pr).TimeOfDay;

                            
                            //Form1.currDataInd.insertData(-1, name,"", price, 0, time, 0, 0, "", 0);
                            Form1.currDataInd.insertData(-1, name, operation, price, vol,time, num, 0, operation, 0);
                        }
                        break;
                    //Данные торгуемого актива
                    /* Настройки таблиц вывода по DDE в Квике:
                     * Рабочая книга - rec
                     * Лист - trade
                     * Столбцы должны располагаться в следующем порядке (строго!):
                     * Номер
                     * Код Бумаги
                     * Операция
                     * Цена
                     * Кол-во
                     * Время
                     * Других столбцов быть не должно!
                     * Снять все галочки, кроме Вывод при нажатии Ctrl+Shift+L
                     */
                    case "[rec]trade":
                        for (int i = 1; i <= rows; i++)
                        {
                            br.ReadUInt16();
                            br.ReadUInt16();
                            num = (int)br.ReadDouble();

                            br.ReadUInt32();
                            name = getString(br);

                            operation = getString(br);

                            br.ReadUInt32();
                            price = br.ReadDouble();

                            vol = br.ReadDouble();

                            br.ReadUInt32();
                            string pr = getString(br);
                            time = Convert.ToDateTime(pr).TimeOfDay;

                            Form1.currDataFut.insertData(-1, name, operation, price, vol, time, num, 0, operation, 0);
                        }
                        break;
                }
                return PokeResult.Processed;
            }
            private string getString(BinaryReader br)
            {
                byte nstr = br.ReadByte();
                byte[] ch = br.ReadBytes(nstr);
                return encoding.GetString(ch, 0, nstr);
            }
        }
    }
}
