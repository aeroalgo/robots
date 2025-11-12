using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading;
using System.Windows.Forms;
using System.Runtime.InteropServices;
using System.Web;
using System.IO;

namespace new_robot_uralpro
{
    //Стартовая форма для запуска робота
    public partial class Form1 : Form
    {
        // Импорт библиотеки для уменьшения кванта времени Windows
        [DllImport("winmm.dll")]
        internal static extern uint timeBeginPeriod(uint period);
        [DllImport("winmm.dll")]
        internal static extern uint timeEndPeriod(uint period);

        public static int nThr = 0;
        static public DateTime dt = DateTime.Now;
        //Прецизионный таймер
        static volatile public System.Diagnostics.Stopwatch timer = new System.Diagnostics.Stopwatch();
        //Флаг запуска коннектора к Плаза2
        public static bool plaza2 = false;
        //Флаг подключения стакана
        public static bool glassP2 = false;
       //Код клиента для Плаза2
        public static string client_code_P2;

        //Структура для записи задаваемых параметров робота (см.ниже расшифровку)
        struct EntryData
        {
            public string futName;
            public string dollName;
            public string classFut;
            public string baseContract;
            public int smaPeriod;
            public int amountFut;
            public int amountFutMax;
            public int normsec;
            public double ks;
            public int Nstock;
            public double koeff;
            public int kTreshold;
            public int priceStep;
            public bool hf;
            public int HFT;
            public string timeEnd;
            public bool nr;
            public int sec;
            public bool th;
        }
        EntryData[] ed = new EntryData[5];
        static object[] cm = new object[5];
        //Структура для записи результатов робота
        public static volatile resultData[] res = new resultData[5];
        Thread[] th = new Thread[5];
        Thread ddeserv;
        Thread ddeclient;
        Thread output;
        Thread thcp2;
        Thread thcp2tr;
        volatile ClassDdeServer ds;
        FormOutput fo;
        //Структура для записи собственных сделок
        public static volatile execord  exec=new execord(40000);
        //Структура для записи коллбэка
        public static volatile callbackTransData cbTrD = new callbackTransData(150000);
        //Структура для записи всех сделок по фьючерсам
        public static volatile currTradesData currDataFut = new currTradesData(1000000);
        //Структура для записи всех сделок по бумагам, входящим в индекс
        public static volatile currTradesData currDataInd = new currTradesData(1000000);
        //public static dataGlassRaw glRaw = new dataGlassRaw(10000000);
        // Класс для формирования стакана
        public static glass GL = new glass(20);
        //Структура для записи транзакций
        public static volatile ComDataTransP2 cdTransP2 = new ComDataTransP2(150000);
        private ClassFunc.TransactionReplyCallback callbackTransactionReply = null;
        private ClassFunc.trade_status_callback trade_callback = null;

        public Form1()
        {
            InitializeComponent(); 
        }

        //Кнопка остановки робота
        public static void btnStop_Clicked(object sender, EventArgs e)
        {
            classDispose(100);
            FormOutput.stopped = true;
        }

       //Кнопка старта робота 
        private void button1_Click(object sender, EventArgs e)
        {
            timeBeginPeriod(1);
            //Считывание из стартовой формы параметров робота
            ClassMain.accountFut = this.textBox8.Text;// номер аккаунта пользователя (для Квика)
            ClassMain.pathToQuick = this.textBox16.Text;//Путь к исполняемому файлу Квика
            ClassMain.USDc = Convert.ToDouble(this.textBox9.Text);//Курс рубля к доллару
            ClassMain.statTime = Convert.ToDateTime(this.textBox10.Text).TimeOfDay;//время открытия амер.рынка
            ClassMain.clearTime = Convert.ToDateTime(this.textBox13.Text).TimeOfDay;//время начала промежуточного клиринга
            System.IO.StreamWriter logwrite = new System.IO.StreamWriter("robot.log", false);//лог робота
            logwrite.Close();

            //nThr - номер потока по порядку
            if (nThr == 0)
            {
                MessageBox.Show("You should enter 1 thread at least");
            }
            else
            {
                //Запуск DDE сервера для Квика - получение цен активов, входящих в индекс (биржа ММВБ)
                ds = new ClassDdeServer();
                ThreadStart ddst = new ThreadStart(ds.mainDdeServer);
                ddeserv=new Thread(ddst);
               
                int result = 0;
                int extendedErrorCode = 0;
                string errorMessage = "";
                //Если Плаза2 не применяется - получение остальных данных по API Квика
                if (!plaza2)
                {
                    this.callbackTransactionReply = new ClassFunc.TransactionReplyCallback(onTransactionReply);// делегат коллбэка
                    result = ClassFunc.SetTransactionReplyCallback(callbackTransactionReply, ref extendedErrorCode, errorMessage,
                        256); //установка делегата на коллбэк Квика по своим ордерам
                    if (result != 0)
                    {
                        MessageBox.Show("Can't set reply callback");
                    }
                    ClassFunc.dllConnect(ClassMain.pathToQuick); //соединение с библиотекой API Квика
                    trade_callback = new ClassFunc.trade_status_callback(trade_status_callback_implementation); //установка делегата на коллбэк Квика по своим сделкам
                    ClassFunc.subscribe_trades("", ""); // подписка на коллбэк по сделкам
                    ClassFunc.start_trades(trade_callback);// запуск отслеживания коллбэков
                    GCHandle gcTrade = GCHandle.Alloc(trade_callback); //выделение дескриптора 
                }

                // Для каждого потока - инициализация переменных основного алгоритма - classMain
                for (int i = 0; i < nThr; i++)
                {
                    res[i] = new resultData(0);
                    if (ed[i].HFT==1)
                    {
                        //Главный алгоритм
                        cm[i] = new ClassMain();
                        ((ClassMain)cm[i]).threadN = i;// номер потока
                        ((ClassMain)cm[i]).futName = ed[i].futName;//имя торгуемого фьючерса
                        ((ClassMain)cm[i]).dollName = ed[i].dollName;// имя валютного фьючерса (Si)
                        ((ClassMain)cm[i]).baseContract = ed[i].baseContract;//базовый контракт фьючерса (RTS)
                        ((ClassMain)cm[i]).classFut = "SPBFUT";//класс торгуемого актива (для QUIK)
                        ((ClassMain)cm[i]).smaPeriod = ed[i].smaPeriod;//Период простой скользящей средней спреда
                        ((ClassMain)cm[i]).amountFut = ed[i].amountFut; // максимальное количество контрактов в торгах
                        ((ClassMain)cm[i]).amountFutMax = ed[i].amountFutMax;//шаг объема (количество контрактов в одной транзакции)
                        ((ClassMain)cm[i]).normsec = ed[i].normsec; //период вычисления среднеквадратичного отклонения
                        ((ClassMain)cm[i]).ks = ed[i].ks; // постоянный коэффициент для определения порога (см.ClassMain)
                        ((ClassMain)cm[i]).koeff = ed[i].koeff; //постоянный коэффициент для вычисления спреда (см.ClassMain)
                        ((ClassMain)cm[i]).kTreshold = ed[i].kTreshold;// постоянный коэффициент при определении цены ордера (см.ClassMain)
                        ((ClassMain)cm[i]).transIDst = i * 50000; //начальное значение номера транзакций
                        int Nind = ed[i].Nstock;//количество бумаг для определения индекса
                        ((ClassMain)cm[i]).indCoeff = new double[Nind]; //коэффициенты для расчета индексов
                        ((ClassMain)cm[i]).indLastPrices = new double[Nind];// массив цен для последних цен индекса
                        ((ClassMain)cm[i]).indNames = new string[Nind];//массив имен бумаг, входящих в индекс
                        ((ClassMain)cm[i]).endTime = Convert.ToDateTime(ed[i].timeEnd).TimeOfDay;//время окончания торговли
                        ((ClassMain)cm[i]).nr = ed[i].nr; //флаг подключения особых условий выхода (см.ClassMain)
                        ((ClassMain)cm[i]).th = ed[i].th; //флаг подключения зависимости цен ордера от волатильности спреда (см.ClassMain)

                        System.IO.StreamReader indread = new System.IO.StreamReader(((ClassMain)cm[i]).futName + ".csv");
                        string[] indarr;

                        //Считывание коэффициентов индекса из файла
                        indarr = indread.ReadLine().Split(';');
                        ((ClassMain)cm[i]).kIndConst = (Convert.ToDouble(indarr[0]) * 100) / (Convert.ToDouble(indarr[1]));// * ClassMain.USDc);
                        double kPart = 0;
                        for (int j = 0; j < Nind; j++)
                        {
                            indarr = indread.ReadLine().Split(';');
                            ((ClassMain)cm[i]).indNames[j] = indarr[0];
                            ((ClassMain)cm[i]).indCoeff[j] = Convert.ToDouble(indarr[1]) * Convert.ToDouble(indarr[2]) * Convert.ToDouble(indarr[3]);
                            kPart += Convert.ToDouble(indarr[4]);
                        }
                        ((ClassMain)cm[i]).kIndConst = ((ClassMain)cm[i]).kIndConst / kPart;
                        indread.Close();

                        //Запуск потоков с алгоритмами
                        th[i] = new Thread(new ThreadStart(((ClassMain)cm[i]).MainSp));
                        th[i].Start();
                    }
                    else if (ed[i].HFT == 2)
                    {
                        //Оставлено для других алгоритмов
                    }
                    else if (ed[i].HFT == 3)
                    {
                        //Оставлено для других алгоритмов
                    }
                }
                base.Dispose(true);
                //Запуск формы контроля и отображения результатов торгов
                fo = new FormOutput();
                output = new Thread(new ThreadStart(fo.formStart));
                output.IsBackground = true;
                output.Priority = ThreadPriority.Lowest;
                //Старт DDE сервера
                ddeserv.Start();
                output.Start();
                GC.KeepAlive(ddeserv);
                //Ожидание окончания работы робота
                output.Join();
                ddeserv.Abort();
                timeEndPeriod(1);
            }          
           
        }

        public static void classDispose(int n)
        {
            if (n == 100)
            {
                for (int i = 0; i < nThr; i++)
                {
                    if (cm[i].GetType() == typeof(ClassMain))
                        ((ClassMain)cm[i]).Dispose();
                }
            }
            else
            {
                if (cm[n].GetType() == typeof(ClassMain))
                    ((ClassMain)cm[n]).Dispose();
            }
        }

        //Приостановка торговли
        public static void classPause(int n,bool ps)
        {
            if (n == 100)
            {
                for (int i = 0; i < nThr; i++)
                {
                    if (cm[i].GetType() == typeof(ClassMain))
                        ((ClassMain)cm[i]).pauseClass=ps;
                }
            }
            else
            {
                if (cm[n].GetType() == typeof(ClassMain))
                    ((ClassMain)cm[n]).pauseClass = ps;
            }
        }

        //Запись данных из хранилищ при сбоях в файл для сохранения
        public static void saveData(string path)
        {
            int nF = 0, nI = 0;
            using (StreamWriter sw = new StreamWriter (path,false))
            {
                while (nF <= currDataFut.pos || nI <= currDataInd.pos)
                {
                    if (nF > currDataFut.pos) currDataFut.timeLoc[nF] = 100000000;
                    if (nI > currDataInd.pos) currDataInd.timeLoc[nI] = 100000000;
                    if ((int)currDataFut.timeLoc[nF] <= (int)currDataInd.timeLoc[nI])
                    {
                        string s1 = Convert.ToString(currDataFut.rev[nF]) + ";" +
                                currDataFut.name[nF] + ";" +
                                currDataFut.operation[nF] + ";" +
                                Convert.ToString(currDataFut.price[nF]).Replace(',', '.') + ";" +
                                Convert.ToString(Convert.ToUInt32(currDataFut.vol[nF])) + ";" +
                                currDataFut.time[nF] + ";" +
                                Convert.ToString(currDataFut.num[nF]) + ";" +
                                Convert.ToString(currDataFut.numOrder[nF]) + ";" +
                                Convert.ToString(currDataFut.myOperation[nF]) + ";" +
                                Convert.ToString(currDataFut.comment[nF]) + ";" +
                                Convert.ToString(dt.TimeOfDay + new TimeSpan(0, 0, 0, 0, (int)currDataFut.timeLoc[nF])) + ";";
                        sw.WriteLine(s1);
                        nF++;
                    }
                    else
                    {
                        string s1 = Convert.ToString(currDataInd.rev[nI]) + ";" +
                                currDataInd.name[nI] + ";" +
                                currDataInd.operation[nI] + ";" +
                                Convert.ToString(currDataInd.price[nI]).Replace(',', '.') + ";" +
                                Convert.ToString(Convert.ToUInt32(currDataInd.vol[nI])) + ";" +
                                currDataInd.time[nI] + ";" +
                                Convert.ToString(currDataInd.num[nI]) + ";" +
                                Convert.ToString(currDataInd.numOrder[nI]) + ";" +
                                Convert.ToString(currDataInd.myOperation[nI]) + ";" +
                                Convert.ToString(currDataInd.comment[nI]) + ";" +
                                Convert.ToString(dt.TimeOfDay + new TimeSpan(0, 0, 0, 0, (int)currDataInd.timeLoc[nI])) + ";";
                        sw.WriteLine(s1);
                        nI++;
                    }
                }
            }
        }

        //Кнопка добавления потоков и считывания данных в форму с записью в массив ed для каждого потока
        private void button2_Click(object sender, EventArgs e)
        {
            ed[nThr].futName = this.textBox1.Text;
            ed[nThr].dollName = this.textBox20.Text;
            ed[nThr].baseContract = this.textBox3.Text;
            ed[nThr].smaPeriod = Convert.ToInt32(this.textBox4.Text);
            ed[nThr].amountFut = Convert.ToInt32(this.textBox5.Text);
            ed[nThr].amountFutMax = Convert.ToInt32(this.textBox6.Text);
            ed[nThr].normsec = Convert.ToInt32(this.textBox7.Text);
            ed[nThr].ks = Convert.ToDouble(this.textBox12.Text);
            ed[nThr].Nstock = Convert.ToInt32(this.textBox15.Text);
            ed[nThr].koeff = Convert.ToDouble(this.textBox17.Text);
            ed[nThr].kTreshold = Convert.ToInt32(this.textBox18.Text);
            ed[nThr].HFT = 1;
            ed[nThr].timeEnd = this.textBox11.Text;
            ed[nThr].nr = checkBox2.Checked;
            ed[nThr].th = checkBox5.Checked;
            string item = String.Format("{0} {1} {2} {3} {4} {5}", ed[nThr].futName, ed[nThr].baseContract, ed[nThr].smaPeriod,
                ed[nThr].amountFut, ed[nThr].amountFutMax, ed[nThr].normsec);

            nThr++;
            this.label15.Text = Convert.ToString(nThr);
            this.listBox1.Items.Add(item);
            
        }

        //Запись коллбэка для API Квика по ордерам
        private void onTransactionReply(int transactionResult, int transactionExtendedErrorCode, int transactionReplyCode,
           uint transactionID, double orderNumber, string transactionReplyMessage)
        {
            string replyMessage = "other";
            cbTrD.cbtime[cbTrD.pos + 1] = Form1.timer.ElapsedMilliseconds;//DateTime.Now;
            cbTrD.transactionResult[cbTrD.pos+1] = transactionResult;
            cbTrD.orderNumber[cbTrD.pos + 1] = orderNumber;
            cbTrD.transactionID[cbTrD.pos+1]=transactionID;
            switch (transactionReplyCode)
            {
                case 3:
                    replyMessage = "Операция выполнена успешно";
                    break;
                case 4:
                    replyMessage = "Не найдена заявка для перестановки";
                    break;
            }
            cbTrD.transactionReplyMessage[cbTrD.pos + 1] = replyMessage;//transactionReplyMessage;
            cbTrD.pos++;
        }

        //Обработка коллбэка для API Квика по собственным сделкам
        private void trade_status_callback_implementation(Int32 nMode, Double dNumber, Double dOrderNumber, string ClassCode,
            string SecCode, Double dPrice, Int32 nQty, Double dValue, Int32 nIsSell, Int32 nTradeDescriptor)
        {
            long t = ClassFunc.TRANS2QUIK_TRADE_TIME(nTradeDescriptor);
            int th = (int)t / 10000;
            int tm = (int)((t - th * 10000) / 100);
            int ts = (int)(t % 100);
            exec.exetime[exec.pos + 1] = new TimeSpan(th, tm, ts);
            exec.exename[exec.pos + 1] = SecCode;
            if (nIsSell == 0) exec.exedir[exec.pos + 1] = "Купля";
            else exec.exedir[exec.pos + 1] = "Продажа";
            exec.exeprice[exec.pos + 1] = dPrice;
            exec.exequan[exec.pos + 1] = (int)nQty;
            exec.execode[exec.pos + 1] = (int)dNumber;
            exec.exeorder[exec.pos + 1] = (int)dOrderNumber;

            string comstr = ClassFunc.TRANS2QUIK_TRADE_BROKERREF(nTradeDescriptor); //поле comment ордера
            if (comstr != "")
            {
                string[] line = comstr.Split('/');
                exec.comment[exec.pos + 1] = Convert.ToInt32(line[line.Length-1]);
            }
            else exec.comment[exec.pos + 1] = -1;
            exec.pos++;
        }

        //Кнопка считывания текущего курса доллара с сайта cbr.ru
        private void button3_Click(object sender, EventArgs e)
        {
            string Val_code = "R01235";
            DateTime DateFrom;
            if (DateTime.Now.DayOfWeek == DayOfWeek.Monday)
            {
                DateFrom = DateTime.Now - new TimeSpan(48, 0, 0);
            }
            else DateFrom = DateTime.Now;
          
            try
            {
                using (ru.cbr.www.DailyInfo di = new ru.cbr.www.DailyInfo())
                {
                    System.Data.DataSet val_ds = di.GetCursDynamic(DateFrom, DateFrom, Val_code);
                    textBox9.Text = Convert.ToString(val_ds.Tables[0].Rows[0][3]);
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show("Error:" + ex.Message, "ERROR");
            }
        }

     
        //Кнопка запуска Плаза2
        private void button4_Click(object sender, EventArgs e)
        {
            timer.Start();
            dt = DateTime.Now;
            plaza2 = checkBox1.Checked;
            glassP2 = checkBox3.Checked;
            client_code_P2 = textBox19.Text.Substring(4, 3);

            ClassP2 cp2 = new ClassP2(textBox19.Text);// класс получения потоков репликации Плаза2
            cp2.nameGlass = this.textBox1.Text;
            thcp2 = new Thread(new ThreadStart(cp2.Run));
            thcp2.SetApartmentState(ApartmentState.MTA);
            thcp2.Start();


            ClassP2Trans cp2tr = new ClassP2Trans(); //класс отправки транзакций Плаза2
            thcp2tr = new Thread(new ThreadStart(cp2tr.Run));
            thcp2tr.SetApartmentState(ApartmentState.MTA);
            thcp2tr.Start();
        }

    }
}
