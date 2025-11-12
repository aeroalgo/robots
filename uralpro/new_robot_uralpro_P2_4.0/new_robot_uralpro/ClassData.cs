using System;
using System.Collections.Generic;
using System.Collections;
using System.Linq;
using System.Text;
using System.IO;

namespace new_robot_uralpro
{
    //Хранилище данных о своих сделках
    public class execord
    {
        public volatile string[] exename;
        public volatile double[] exeprice;
        public volatile int[] exequan;
        public volatile string[] exedir;
        public volatile int[] comment;
        public volatile TimeSpan[] exetime;
        public volatile long[] execode;
        public volatile long[] exeorder;
        public volatile int pos;
        public execord (int sz)
        {
            pos = -1;
            exename = new string[sz];
            exeprice = new double[sz];
            exequan = new int[sz];
            exedir = new string[sz];
            comment = new int[sz];
            exetime = new TimeSpan[sz];
            execode = new long[sz];
            exeorder = new long[sz];
        }
        public void insertData(string stnameKey, string operationKey, double priceKey, double amountKey, TimeSpan timeKey, long numKey, long numOrderKey, int commentKey)
        {
            int npos = pos + 1;
            exename[npos] = stnameKey;
            exedir[npos] = operationKey;
            exeprice[npos] = priceKey;
            exequan[npos] = (int)amountKey;
            exetime[npos] = timeKey;
            execode[npos] = numKey;
            exeorder[npos] = numOrderKey;
            comment[npos] = commentKey;
            pos++;
        }
        public long readData(currTradesData cdAE)
        {
            //if (pos == -1)
            //{
            for (int i = 0; i <= cdAE.pos; i++)
            {
                if (cdAE.rev[i] != -1)
                {
                    if (cdAE.numOrder[i] != 0)
                    {
                        pos++;
                        exename[pos] = cdAE.name[i];
                        exedir[pos] = cdAE.myOperation[i];
                        exeprice[pos] = cdAE.price[i];
                        exequan[pos] = (int)cdAE.vol[i];
                        exetime[pos] = cdAE.time[i];
                        execode[pos] = cdAE.num[i];
                        exeorder[pos] = cdAE.numOrder[i];
                        comment[pos] = cdAE.comment[i];
                    }
                }
            }
            //}
            return 0;
        }
    }
    //Класс формирования стакана
    public struct glass
    {
        public double[] bid;
        public double[] ask;
        public double[] volAsk;
        public double[] volBid;
        public long[] replIDbid;
        public long[] replIDask;
        public TimeSpan glassTime;

        public glass(int sz)
        {
            bid = new double[sz];
            ask = new double[sz];
            replIDbid = new long[sz];
            replIDask = new long[sz];
            volAsk = new double[sz];
            volBid = new double[sz];
            glassTime = new TimeSpan(0, 0, 0);
        }
    }
    //Хранилище данных стакана с Плаза2
    public class dataGlassRaw
    {
        public int[] replID;
        public string[] name;
        public double[] price;
        public int[] dir;
        public double[] vol;
        public TimeSpan[] glTimeRTS;
        public int pos;

        public dataGlassRaw(int sz)
        {
            replID = new int[sz];
            name = new string[sz];
            price = new double[sz];
            dir = new int[sz];
            vol = new double[sz];
            glTimeRTS = new TimeSpan[sz];
            pos = -1;
        }
        public void saveData(string path)
        {
            if (pos >= 0)
            {
                using (StreamWriter sw = new StreamWriter(path, false))
                {
                    for (int i = 0; i <= pos; i++)
                    {
                        string s1 = Convert.ToString(replID[i]) + ";" +
                            name[i] + ";" +
                            Convert.ToString(price[i]).Replace(',', '.') + ";" +
                            Convert.ToString(dir[i]) + ";" +
                            Convert.ToString(Convert.ToUInt32(vol[i])) + ";" +
                            glTimeRTS[i] + ";";
                        sw.WriteLine(s1);
                    }
                }
            }
        }
    }
    //не применяется
    public struct execData
    {
        public double[] price;
        public double[] vol;
        public int[] dir;
        public TimeSpan[] time;
        public int pos;
        public execData(int sz)
        {
            price = new double[sz];
            vol = new double[sz];
            time = new TimeSpan[sz];
            dir = new int[sz];
            pos = -1;
        }
    }
    //Хранилище коллбэков
    public class callbackTransData
    {
        public volatile uint[] transactionID;
        public volatile double[] orderNumber;
        public volatile string[] transactionReplyMessage;
        public volatile int[] transactionResult;
        public volatile long[] cbtime;
        public volatile int[] delamount;
        public volatile int pos;
        //private static object lockInsertData = new object();
        public callbackTransData(int sz)
        {
            pos = -1;
            transactionID = new uint[sz];
            orderNumber = new double[sz];
            transactionReplyMessage = new string[sz];
            transactionResult = new int[sz];
            cbtime = new long[sz];
            delamount=new int[sz];
        }
        public void saveData(string path)
        {
            if (pos >= 0)
            {
                using (StreamWriter sw = new StreamWriter(path, false))
                {
                    for (int i = 0; i <= pos; i++)
                    {
                        string time = Form1.dt.AddMilliseconds(cbtime[i]).ToString("hh:mm:ss.FFF");
                        string s1 = String.Format("{0}  ordN:{1}    {2} result:{3}  delam:{4}   {5}",
                            transactionID[i], orderNumber[i], transactionReplyMessage[i], transactionResult[i], delamount[i], time);
                        sw.WriteLine(s1);
                    }
                }
            }
        }
        public void insertData(uint trID, double oN, string trReply, int trRes, long cbt, int delam)
        {
            //lock (lockInsertData)
            //{
                int npos = pos + 1;
                transactionID[npos] = trID;
                orderNumber[npos] = oN;
                transactionReplyMessage[npos] = trReply;
                transactionResult[npos] = trRes;
                cbtime[npos] = cbt;
                delamount[npos] = delam;
                pos++;
            //}
        }
    }
    //Хранилище данных всех сделок
    public class currTradesData
    {
        public volatile long[] rev;
        public volatile TimeSpan[] time;
        public volatile string[] name;
        public volatile double[] price;
        public volatile double[] vol;
        public volatile string[] operation;
        public volatile long[] num;
        public volatile long[] numOrder;
        public volatile string[] myOperation;
        public volatile int[] comment;
        public volatile int pos;
        public volatile double[] timeLoc;

        long lastrev;
        //TimeSpan lasttime = new TimeSpan(10, 0, 0);
        //private object lockInsertData = new object();

        public currTradesData(int sz)
        {
            pos = -1;
            rev=new long[sz];
            time = new TimeSpan[sz];
            name = new string[sz];
            price = new double[sz];
            vol = new double[sz];
            operation = new string[sz];
            num=new long[sz];
            numOrder=new long[sz];
            myOperation = new string[sz];
            comment=new int[sz];
            timeLoc = new double[sz];
        }

        public void insertData(long revKey, string stnameKey, string operationKey, double priceKey, double volumeKey, TimeSpan timeKey, long numKey, long numOrderKey, string myOperationKey, int commentKey)
        {
            //lock (lockInsertData)
            //{
                if (revKey == -1 || revKey > lastrev)
                {
                    int npos = pos + 1;
                    rev[npos] = revKey;
                    name[npos] = stnameKey;
                    operation[npos] = operationKey;
                    price[npos] = priceKey;
                    vol[npos] = volumeKey;
                    //if (timeKey.Hours == 0) time[npos] = lasttime;
                    //else
                    //{
                    time[npos] = timeKey;
                        //lasttime = timeKey;
                    //}
                    num[npos] = numKey;
                    numOrder[npos] = numOrderKey;
                    myOperation[npos] = myOperationKey;
                    comment[npos] = commentKey;
                    timeLoc[npos] = Form1.timer.ElapsedMilliseconds;
                    pos++;
                    if (revKey != -1) lastrev = revKey;
                }
            //}
        }

        public void saveData(string path)
        {
            if (pos >= 0)
            {
                using (StreamWriter sw = new StreamWriter(path,false))
                {
                    for (int i = 0; i <= pos; i++)
                    {
                        TimeSpan tL = (Form1.dt.TimeOfDay + new TimeSpan(0, 0, 0, 0, (int)timeLoc[i]));
                        string s1 = Convert.ToString(rev[i]) + ";" +
                            name[i] + ";" +
                            operation[i] + ";" +
                            Convert.ToString(price[i]).Replace(',', '.') + ";" +
                            Convert.ToString(Convert.ToUInt32(vol[i])) + ";" +
                            time[i] + ";" +
                            Convert.ToString(num[i]) + ";" +
                            Convert.ToString(numOrder[i]) + ";" +
                            Convert.ToString(myOperation[i]) + ";" +
                            Convert.ToString(comment[i]) + ";"+
                            tL.ToString()+";";
                        sw.WriteLine(s1);
                    }
                }
            }
        }
        public long readData(string path)
        {
            if (File.Exists(path))
            {
                using (StreamReader sr = new StreamReader(path))
                {
                    while (true)
                    {
                        string s = sr.ReadLine();
                        if (s == null || s == "") break;
                        string[] sarray = s.Split(';');
                        if (Convert.ToInt32(sarray[0]) != -1)
                        {
                            pos++;
                            rev[pos] = Convert.ToInt32(sarray[0]);
                            if (rev[pos] != -1) lastrev = rev[pos];
                            name[pos] = sarray[1];
                            operation[pos] = sarray[2];
                            price[pos] = (float)Convert.ToDouble(sarray[3].Replace('.', ','));
                            vol[pos] = (float)Convert.ToDouble(sarray[4]);
                            time[pos] = Convert.ToDateTime(sarray[5]).TimeOfDay;
                            num[pos] = Convert.ToInt64(sarray[6]);
                            numOrder[pos] = Convert.ToInt64(sarray[7]);
                            myOperation[pos] = sarray[8];
                            comment[pos] = Convert.ToInt32(sarray[9]);
                        }
                    }
                }
                return lastrev;
            }
            return 0;
        }
    }
    //Структура параметров и результатов робота
    public struct resultData
    {
        public double summ;
        public double sumMax;
        public double sumMin;
        public int futpoz;
        public double delay;
        public TimeSpan time;
        public int contracts;
        public int trans;
        public int trades;
        public int err;
        public string id;
        public resultData(int n)
        {
            summ = 0;
            sumMax = 0;
            sumMin = 0;
            futpoz = 0;
            delay = 0;
            time = new TimeSpan(10, 30, 0);
            contracts = 0;
            trans = 0;
            trades = 0;
            err = 0;
            id = "";
        }
    }
    //Хэш таблица для связывания кодов и имен фьючерсов - для Плаза2
    public class ComDatafutInfo
    {
        public volatile int[] isin_id;
        public volatile string[] short_isin;
        public volatile string[] isin;
        Hashtable dict_isin_id_TO_short_isin;
        Hashtable dict_short_isin_TO_isin;
        public volatile int pos;

        public ComDatafutInfo(int n)
        {
            isin_id = new int[n];
            short_isin = new string[n];
            isin = new string[n];
            pos = -1;
        }
        public void createHash()
        {
            dict_isin_id_TO_short_isin = new Hashtable();
            dict_short_isin_TO_isin = new Hashtable();
            for (int i = 0; i <= pos; i++)
            {
                dict_isin_id_TO_short_isin.Add(isin_id[i], short_isin[i]);
                dict_short_isin_TO_isin.Add(short_isin[i], isin[i]);
            }
        }
        public object convertIsinIdToShortIsin(int i)
        {
            return dict_isin_id_TO_short_isin[i];
        }
        public object convertShortIsinToIsin(string s)
        {
            return dict_short_isin_TO_isin[s];
        }
        public void readFromFile(string path)
        {
            using (StreamReader sr = new StreamReader(path))
            {
                while (sr.Peek() >= 0)
                {
                    string s = sr.ReadLine();
                    if (s == "") break;
                    string[] sarray = s.Split(';');
                    //if (Convert.ToInt32(sarray[7]) != 4)
                    //{
                    pos++;
                    isin_id[pos] = Convert.ToInt32(sarray[0]);
                    short_isin[pos] = sarray[1];
                    isin[pos] = sarray[2];
                    //}
                }
            }
        }
    }
    //Хранилище транзакций Плаза2
    public class ComDataTransP2
    {
        public volatile int[] amTrans;
        public volatile int[] p2type;
        public volatile long[] ordNum;
        public volatile string[] stname;
        public volatile int[] operation;
        public volatile double[] price;
        public volatile double[] volume;
        public volatile string[] comment;
        public volatile long[] trTime;
        public volatile int pos;

        public ComDataTransP2(int n)
        {
            amTrans = new int[n];
            p2type = new int[n];
            ordNum = new long[n];
            stname = new string[n];
            operation = new int[n];
            price = new double[n];
            volume = new double[n];
            comment = new string[n];
            trTime = new long[n];
            pos = -1;
        }
        public void saveData(string path)
        {
            if (pos >= 0)
            {
                using (StreamWriter sw = new StreamWriter(path, false))
                {
                    for (int i = 0; i <= pos; i++)
                    {
                        string time = Form1.dt.AddMilliseconds(trTime[i]).ToString("hh:mm:ss.FFF");
                        string s1 = String.Format("{0}  P2Type:{1}  ordN:{2}    oper:{3}    price:{4}   am:{5}  comm:{6}    {7}",
                            amTrans[i], p2type[i], ordNum[i], operation[i], price[i], volume[i], comment[i], time);
                        sw.WriteLine(s1);
                    }
                }
            }
        }
        private static object lockOrder = new object();
        public void DelOrder(long ordNumKey, int amTransKey)
        {
            lock (lockOrder)
            {
                int npos = pos + 1;
                amTrans[npos] = amTransKey;
                p2type[npos] = 2;
                ordNum[npos] = ordNumKey;
                pos++;
                //curAmTransKey = amTransKey;
                //amTransKey++;
            }
        }
        public void addOrder(string stnameKey, int operationKey, double priceKey, double volumeKey, string commentKey, int amTransKey)
        {
            lock (lockOrder)
            {
                int npos = pos + 1;
                amTrans[npos] = amTransKey;
                p2type[npos] = 1;
                stname[npos] = stnameKey;
                operation[npos] = operationKey;
                price[npos] = priceKey;
                volume[npos] = volumeKey;
                comment[npos] = commentKey;
                pos++;
                //curAmTransKey = amTransKey;
                //amTransKey++;
            }
        }

        public void moveOrder(long ordNumKey, double priceKey, int amTransKey)
        {
            lock (lockOrder)
            {
                int npos = pos + 1;
                amTrans[npos] = amTransKey;
                p2type[npos] = 5;
                ordNum[npos] = ordNumKey;
                price[npos] = priceKey;
                pos++;
                //curAmTransKey = amTransKey;
                //amTransKey++;
            }
        }
    }
}
