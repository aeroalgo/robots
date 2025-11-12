using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Windows.Forms;
namespace new_robot_uralpro
{
    // Основной класс алгоритма
    class ClassMain 
    {
        public string futName;
        public static string accountFut;
        public string baseContract;
        public string classFut;
        public string dollName;
        public int smaPeriod;
        public int normsec;
        public int amountFut;
        public int amountFutMax;
        public bool nr;
        public bool th;
        public double ks;
        public static double USDc;
        public static TimeSpan statTime,clearTime;
        public TimeSpan endTime;
        //минимальный шаг цены фьючерса
        public int priceStep=5;
        public static string pathToQuick;
        public int transIDst;
        public string[] indNames;
        public double[] indCoeff;
        public double[] indLastPrices;
        public double koeff;
        public int kTreshold;
        public double kIndConst;
        public int threadN;
        public bool pauseClass = false;
        bool Stopped = false;
        bool bCB = true, sCB = true;
        bool bNF = true, sNF = true;
        int bWait = 0, sWait = 0;
        int bTransId = 0, sTransId = 0;
        long bDel, sDel;
        int sDelint = 0, bDelint = 0;
        double priceBuyFut = 0, priceSellFut = 0;
        double priceBuyFutPrev = 0, priceSellFutPrev = 0;
        double priceBuyFutSave = 0, priceSellFutSave = 0;
        long oN1 = 0, oN2 = 0;
        int buyCount = 0, sellCount = 0, buyCountSave = 0, sellCountSave = 0;
        int buyErr = 0, sellErr = 0;
        //размер массива задержек для расчета среднего раундтрипа
        int delayCount = 20;
        double[] delay;
        int delayi = 0;
        double delaySec = 0;

        public void MainSp()
        {
            int transID = transIDst;
            TimeSpan timeSpread = new TimeSpan(10,30,0);
            int fastbs = 0;
            int exbs = 3;
            int currTime = 0, currTimePrev = 0, futTime = 0, stTime = 0;
            //Время начала торговли
            TimeSpan timeBegin = new TimeSpan(10, 35, 0);
            int timeStart =(int)timeBegin.TotalSeconds;
            int keyValFut = 0, keyValInd = 0;
            double kbs = 0;
            double summ = 0;
            double summin = 0, summax = 0;
            double summPrev = 0;
            double spreadnorm = 0;
            int buyAmountFut = 0, sellAmountFut = 0;
            int futPozClear = 0, futPozBuy = 0, futPozSell = 0;
            int futPozBuyPrev = 0, futPozSellPrev = 0;
            double summTotal=0;
            double dollLastPrice = ClassMain.USDc;
            int execTableCode = -1;
            int contracts = 0;
            int cbCode = -1;
            //время максимального ожидания коллбэка в секундах
            int delayCBmax = 120;
            int trades=0;
            int errors=0;
            double[] prSt = new double[67000];
            double[] prFut = new double[67000];
            double[] volF = new double[67000];
            double[] volFN = new double[67000];
            double[] spread = new double[67000];
            double[] nspread = new double[67000];
            double[] sma = new double[67000];
            int[] freqB = new int[67000];
            int[] freqS = new int[67000];
            delay = new double[delayCount];
            long[] ordActArr = new long[20];
            int ordActArrPos = -1;
            int saveResT = 0;
            double sumBS = 0;

            int timeClear = 0, timeEnd = 0, timeStat = 0;

            if (nr) Form1.res[threadN].id = String.Format("{0}_{1} ({2},{3},{4})_nr", futName, threadN, smaPeriod, amountFut, amountFutMax);
            else Form1.res[threadN].id = String.Format("{0}_{1} ({2},{3},{4})", futName, threadN, smaPeriod, amountFut, amountFutMax);

            //Считывание текущего времени из хранилища сделок
            timeSpread = Form1.currDataFut.time[Math.Max(Form1.currDataFut.pos, 0)];
            // Ожидание заданного времени старта
            while (timeSpread.TotalSeconds <= timeStart-10)
            {
                timeSpread = Form1.currDataFut.time[Math.Max(Form1.currDataFut.pos, 0)];
                System.Threading.Thread.Sleep(500);
                if (Stopped) break;
            }
            if (timeSpread.TotalSeconds > timeStart-10)
            {
                System.Threading.Thread.Sleep(10000);
            }
            // Начало основного цикла
            while (!Stopped)
            {
                // Цикл,в котором вычисляются текущие значения и дается сигнал на покупку или продажу
                do
                {
       
                    System.Threading.Thread.Sleep(1);
                    /* Считывание основных параметров ( см. комментарии в ClassFunc):
                     * futTime - текущее время торгуемого фьючерса (RI)
                     * stTime - текущее время индекса
                     * prFut - массив цен торгуемого фьючерса - посекундный
                     * prSt - массив цен индекса посекундный
                     * indLastPrices - текущие цены активов, входящих в индекс
                     * indCoeff - постоянные коэффициенты активов в индексе (доли участия)
                     * volF - массив объемов покупок фьючерса посекундный
                     * volFN - массив объемов продаж фьючерса посекундный
                     * indNames - массив имен активов, входящих в индекс
                     * kIndConst - константа, учавсвующая в расчете индекса
                     * keyValFut, keyValInd - указатели на конец хранилищ данных для фьючерса и индекса соответственно
                     * dollName - имя валютного фьючерса
                     * dollLastPrice - текущая цена валютного фьючерса
                     * freqB,freqS  - посекундные массивы числа покупок и продаж торгуемого фьючерса соответственно
                     */
                    if (ClassFunc.DBReadI2(Form1.currDataFut,Form1.currDataInd, futName, ref futTime, ref stTime, ref timeStart, ref prFut,
                        ref prSt,ref indLastPrices, indCoeff,ref volF,ref volFN,indNames,kIndConst,ref keyValFut, ref keyValInd, dollName,
                        ref dollLastPrice,ref freqB,ref freqS))
                    {
                        // Задаем время клиринга, окончания торговли и timeStat - время начала торгов на американских биржах
                        //(для обхода гэпа на открытии), в целочисленной форме - секунды со старта торгов
                        if (currTime == 0)
                        {
                            timeClear = (int)clearTime.TotalSeconds - timeStart;
                            timeEnd = (int)endTime.TotalSeconds - timeStart;
                            timeStat = (int)statTime.TotalSeconds - timeStart;
                        }

                        // Приведение к единому времени currTime фьючерса и индекса, и соответственное приведение массивов цен
                        currTimePrev = Math.Min(Math.Min(futTime, stTime), currTime);
                        currTime = Math.Max(futTime, stTime);
                        for (int i = stTime + 1; i <= currTime; i++)
                        {
                            prSt[i] = prSt[stTime];
                        }
                        
                        for (int i = futTime + 1; i <= currTime; i++)
                        {
                            prFut[i] = prFut[futTime];
                        }

                        // Вычисление текущего спреда - разницы цен фьючерса и индекса, koeff - постоянный заданный коэффициент
                        // spread - посекудный массив мгновенного спреда, sma - простая скользящая средняя спреда с периодом smaPeriod
                        // nspread - посекундный массив спреда с обнуленной трендовой составляющей
                        for (int i = currTimePrev; i <= currTime; i++)
                        {
                            spread[i] = Math.Log(prFut[i]) - koeff * Math.Log(prSt[i]);
                            sma[i] = ClassFunc.timeInt(smaPeriod, i, spread);
                            nspread[i] = spread[i] - sma[i];
                        }
                        //Среднеквадратичное значение спреда
                        spreadnorm = ClassFunc.timeIntS(normsec, nspread, currTime);
                        //Коэффициенты, отражающие соотношение объемов покупок/продаж (рассчитываются за последние 60 сек)
                        double vb = ClassFunc.timeInt(60, currTime, volF);
                        double vs = ClassFunc.timeInt(60, currTime, volFN);
                        if (vs == 0) vs = 0.001;
                        else kbs = vb / vs;

                        // nr - булевая переменная, определяющая, подключать ли дополнительные условия выхода из открытой позиции:
                        // exbs=1 - выход из короткой позиции, 2 - выход из длинной позиции. Выход инициируется, если происходит всплеск
                        // в покупках или продажах, определяемый коэффициентами sumB, sumS и sumBS.
                        if (nr)
                        {
                            int sec = 1;
                            int sumB = 0;
                            int sumS = 0;
                            while (sumB + sumS < 50 && currTime > 10)
                            {
                                sumB = ClassFunc.Summ(sec, currTime, freqB);
                                sumS = ClassFunc.Summ(sec, currTime, freqS);
                                sec++;
                            }

                            if (sumS != 0) sumBS = (double)sumB / (double)sumS;
                            else sumBS = 100;

                            if (sumS > sumB)  exbs = 2;
                            else if (sumS < sumB) exbs = 1;
                            else exbs = 3;
                        }
                    }
                    timeSpread =new TimeSpan(0, 0, currTime+timeStart);
                    // Окончание работы по времени или команде, изменяющей флаг Stopped
                    if (Stopped || currTime > timeEnd+120)
                    {
                        Stopped = true;
                        break;
                    }

                    // Определение сигнала (fastbs) на покупку или продажу - если нормированный текущий спред выше порога (ks - задаваемый постоянный
                    // коэффициент - величина порога) - продажа, если ниже порога - покупка, иначе - цикл продолжает работу
                    if (currTime > 1)
                    {
                        
                        if (nspread[currTime] >= ks * spreadnorm)
                            fastbs = 2;
                        else if (nspread[currTime] <= -ks * spreadnorm)
                            fastbs = 1;
                        else fastbs = 3;
                        
                    }

                } while (fastbs==3);

                //Объем ордера на покупка и ордера на продажу соответственно
                buyAmountFut = 0;
                sellAmountFut = 0;
                // Считывание коллбэка
                onTransactionReply(Form1.cbTrD, ref cbCode);
                /* Считывание результатов собственных сделок:
                 * futPozBuy, futPozSell - совокупные длинная и короткая позиции
                 * summTotal - финансовый результат торговли (в пунктах)
                 * contracts - общее количество проданных и купленных контрактов
                 * trades - общее количество сделок
                 * threadN - номер данного потока ( нужен, если запускается несколько параллельных потоков)
                 */
                ClassFunc.DBReadExec(Form1.exec, futName, ref futPozBuy, ref futPozSell, ref summTotal, ref execTableCode,
                  ref contracts, ref trades, threadN);
                // Вычисление открытой позиции
                futPozClear = futPozBuy - futPozSell;

                //Обнуление переменных при первом запуске
                if (transID == transIDst)
                {
                    futPozBuyPrev = futPozBuy;
                    futPozSellPrev = futPozSell;
                    buyCount = 0;
                    sellCount = 0;
                }

                // Текущий финансовый результат с учетом открытой позиции
                summ = summTotal + futPozClear * prFut[currTime];

                // Служебные переменные, необходимые для отслеживания состояния ордеров и контроля над их выставлением и передвижением
                int buyCh = futPozBuy - futPozBuyPrev;//изменение позиции по купленным контрактам
                int sellCh = futPozSell - futPozSellPrev;//изменение позиции по проданным контрактам
                futPozBuyPrev = futPozBuy;
                futPozSellPrev = futPozSell;
                buyCount = buyCount - buyCh;//вычитание сработавших ордеров на покупку из выставленных
                sellCount = sellCount - sellCh;//вычитание сработавших ордеров на продажу из выставленных
                if (buyCount <= 0) bNF = true;//отмена блокировки выставления по ордерам на покупку
                if (sellCount <= 0) sNF = true;//отмена блокировки выставления по ордерам на продажу

                // Вычисление цены для ордеров на покупку и продажу
                #region priceCalc
                //dob - прибавка к цене покупки или продажи в зависимости от состояния рынка. Здесь задается начальное значение dob, равное минимальному расстоянию от текущей цены
                //kTreshhold - постоянная, задающая начальное расстояние от текущей цены.
                // smax - расстояние от текущей цены для противоположного сигналу fastbs ордера
                int dob = kTreshold*priceStep, smax = 30*priceStep;
                //Пороги, задающие, при каком изменении текущей цены передвигать ордер со старой позиции (для покупки и продажи отдельно)
                int bPor = 0, sPor = 0;

                // th - флаг, задающий применение  условия зависимости цены от волатильности спреда (зависит от состояния рынка, решение принимается при запуске робота, на основании прошлых сессий)
                // Условие такое: при превышении спредом определенной величины порога цена ордера на стороне сигнала приближается к текущей цене
                // чем больше размах спреда, тем больше приближение. priceStep - минимальный шаг цены фьючерса
                if (!th)
                {
                    if (Math.Abs(nspread[currTime]) >= 3.0 * spreadnorm) dob -= 6 * priceStep;
                    else if (Math.Abs(nspread[currTime]) >= 2.5 * spreadnorm) dob -= 5 * priceStep;
                    else if (Math.Abs(nspread[currTime]) >= 2.0 * spreadnorm) dob -= 4 * priceStep;
                    else if (Math.Abs(nspread[currTime]) >= 1.5 * spreadnorm) dob -= 3 * priceStep;
                    else if (Math.Abs(nspread[currTime]) >= 1.0 * spreadnorm) dob -= 2 * priceStep;
                    else if (Math.Abs(nspread[currTime]) >= 0.5 * spreadnorm) dob -= priceStep;
                }

                // Вычисление цен ордеров в зависимости от сигнала fastbs
                if (currTime > 2)
                {
                    // Покупка
                    if (fastbs==1)
                    {
                        if (!th)
                        {
                            //Цена ордера на стороне сигнала приближается к текущей цене в зависимости от изменения текущей цена за последние 2 секунды
                            if (prFut[currTime] - prFut[currTime - 2] >= 4 * priceStep) dob -= priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] >= 6 * priceStep) dob -= priceStep;

                            if (prFut[currTime] - prFut[currTime - 2] <= -priceStep) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] <= -(2 * priceStep)) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] <= -(3 * priceStep)) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] <= -(4 * priceStep)) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] <= -(5 * priceStep)) dob += priceStep;

                            // Добавка к цене, зависящая от соотношения объемов покупки/продажи
                            if (kbs > 1) dob += priceStep;
                            else if (kbs < 1) dob -= priceStep;
                        }
                        // Установка цены ордеров на покупку и продажу и порогов передвижения для текущего сигнала
                        // prFut[currTime] - текущая цена
                        priceBuyFut = prFut[currTime] - dob;
                        priceSellFut = prFut[currTime] + smax;
                        bPor = 3 * priceStep; sPor = 10 * priceStep;
                    }
                    else if (fastbs==2)
                    {
                        //Продажа. Все аналогично, как при покупке
                        if (!th)
                        {
                            if (prFut[currTime] - prFut[currTime - 2] <= -(4 * priceStep)) dob -= priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] <= -(6 * priceStep)) dob -= priceStep;

                            if (prFut[currTime] - prFut[currTime - 2] >= priceStep) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] >= 2 * priceStep) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] >= 3 * priceStep) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] >= 4 * priceStep) dob += priceStep;
                            if (prFut[currTime] - prFut[currTime - 2] >= 5 * priceStep) dob += priceStep;


                            if (kbs < 1) dob += priceStep;
                            else if (kbs > 1) dob -= priceStep;
                        }

                        priceSellFut = prFut[currTime] + dob;
                        priceBuyFut = prFut[currTime] - smax;
                        sPor = 3 * priceStep; bPor = 10 * priceStep;
                    }

                    // Дополнительное условие выхода при позиции, открытой в противоположную сторону предполагаемого движения рынка.
                    if (nr)
                    {
                        if (exbs == 2 && futPozClear > 0)
                        {
                            // Ожидается снижение цены
                            fastbs = 2;
                            // Приближение цены продажи к текущей цене
                            if (sumBS <= 0.125) priceSellFut = Math.Min(prFut[currTime] + 4*priceStep,priceSellFut);
                            else priceSellFut = Math.Min(prFut[currTime] + 5*priceStep,priceSellFut);
                            // Удаление цены покупки от текущей
                            priceBuyFut = prFut[currTime] - smax;
                            sPor = 3 * priceStep; bPor = 10 * priceStep;

                        }
                        else if (exbs == 1 && futPozClear < 0)
                        {
                            //Ожидается рост цены
                            fastbs = 1;
                            if (sumBS >= 8) priceBuyFut = Math.Max(prFut[currTime] - 4*priceStep,priceBuyFut);
                            else priceBuyFut = Math.Max(prFut[currTime] - 5*priceStep,priceBuyFut);
                            priceSellFut = prFut[currTime] + smax;
                            bPor = 3 * priceStep; sPor = 10 * priceStep;

                        }
                    }
                }
                #endregion

                // Контроль ошибок - если не вернулся коллбэк за определенное время
                if (buyErr >= 50 || sellErr >= 50)
                {
                    string errTime = Form1.dt.AddMilliseconds(Form1.timer.ElapsedMilliseconds-delayCBmax*1000).ToString("hh:mm:ss.FFF");
                    string s = "ERROR  " + errTime;
                    //Запись ошибки в лог
                    ClassP2Trans.LogWriteLine(s);
                    System.Threading.Thread.Sleep(15000);

                    // Считывание активных ордеров, находящихся в рынке, и их удаление
                    do
                    {
                        if (Form1.plaza2)
                        {
                            ClassP2.readActiveOrder(ref ordActArr, ref ordActArrPos, Convert.ToString(threadN));
                        }
                        else
                        {
                            ClassFunc.readOrder(ref ordActArr, ref ordActArrPos, Convert.ToString(threadN));
                        }
                        if (ordActArrPos >= 0)
                        {
                            for (int i = 0; i <= ordActArrPos; i++)
                            {
                                transID++;
               
                                if (Form1.plaza2)
                                {
                                    Form1.cdTransP2.DelOrder(ordActArr[i], transID);
                                }
                                else
                                {
                                    ClassFunc.killFutOrder(transID, accountFut, baseContract, 'N', futName, ordActArr[i]);
                                }
                                System.Threading.Thread.Sleep(500);
                            }
                        }
                    } while (ordActArrPos >= 0);

                    // Обнуление служебных счетчиков
                    buyCount = 0; sellCount = 0;
                    buyErr = 0; sellErr = 0;
                    // Увеличиваем счетчик ошибок для информирования
                    errors++;
                }

                // Отправка ордеров на биржу
                #region orderSend
                //Добавление ордера (ADD)
                if (buyCount <= 0) // если нет выставленных ордеров на покупку - разрешаем выставление
                {
                    bNF = true; // отмена блокировки
                    // Покупка
                    if (fastbs==1)
                    {
                        // Если до конца торговли менее 5 минут - закрываем открытую позицию в 0
                        if (currTime > timeEnd - 300 || pauseClass)
                        {
                            buyAmountFut = Math.Max(0, -futPozClear);
                        }
                        // Закрываем позиции перед клирингом и открытием западных рынков, чтобы избежать гэпов
                        else if ((currTime > timeClear - 300 && currTime < timeClear)
                            || (currTime > timeStat - 180 && currTime < timeStat + 180))
                        {
                            buyAmountFut = Math.Max(0, -futPozClear);
                        }
                        else if (currTime >= timeClear && currTime <= timeClear + 240)
                        {
                            buyAmountFut = 0;
                        }
                        // Установка объема ордера на покупку
                        //amountFut - максимальное количество контрактов, заданное для торгов
                        //amountFutMax - шаг объема
                        else
                        {
                            buyAmountFut = Math.Min(amountFutMax, amountFut - futPozClear);
                        }

                        //Отправка ордера с установленными параметрами на биржу
                        if (bCB && buyAmountFut > 0)
                        {
                            //Номер транзакции
                            transID++;
                            bTransId = transID; //запоминания номера транзакции для коллбэка
                            bWait = 0; //признак добавления ордера

                            int opInt = 1;
                            
                            if (Form1.plaza2)
                            {
                                Form1.cdTransP2.addOrder(futName, opInt, priceBuyFut, buyAmountFut, Convert.ToString(threadN), transID);
                            }
                            else
                            {
                                ClassFunc.putFutOrder(transID, classFut, futName, 'B', buyAmountFut, priceBuyFut, accountFut, threadN);
                            }
                            //Фиксация времени отправки (в миллисекундах и секундах)
                            bDel = Form1.timer.ElapsedMilliseconds;//
                            bDelint = currTime;
                            
                            bCB = false; // флаг ожидания коллбэка ордера на покупку
                            priceBuyFutSave = priceBuyFut; //запоминания цены ордера до подтверждения по коллбэку
                            buyCountSave = buyAmountFut;// запоминание объема ордера до подтверждения по коллбэку
                        }
                    }
                }
                //Передвижение существующего ордера (MOVE)
                else if (Math.Abs(priceBuyFut - priceBuyFutPrev)>=bPor)
                {
                    if (bCB && bNF) // если нет ожидания коллбэка и нет блокировки по передвижению
                    {
                        transID++;
                        bTransId = transID;
                        bWait = 1;// признак передвижения ордера
                        
                        if (Form1.plaza2)
                        {
                            Form1.cdTransP2.moveOrder(oN1, priceBuyFut, transID);
                        }
                        else
                        {
                            ClassFunc.MoveOrder(classFut, futName, buyAmountFut, 0, priceBuyFut, priceSellFut, oN1, 0,
                              transID, 0);
                        }

                        bDel = Form1.timer.ElapsedMilliseconds;//
                        bDelint = currTime;
                       
                        bCB = false;
                        priceBuyFutSave = priceBuyFut;
                    }
                }
                
                //Продажа. Все аналогично, как для покупки
                if (sellCount <= 0)
                {
                    sNF = true;
                    if (fastbs==2)
                    {
                        if (currTime > timeEnd - 300 || pauseClass)
                        {
                            sellAmountFut = Math.Max(0, futPozClear);
                        }
                        else if ((currTime > timeClear - 300 && currTime < timeClear)
                             || (currTime > timeStat - 180 && currTime < timeStat + 180))
                        {
                            sellAmountFut = Math.Max(0, futPozClear);
                        }
                        else if (currTime >= timeClear && currTime <= timeClear + 240)
                        {
                            sellAmountFut = 0;
                        }
                        else
                        {
                            sellAmountFut = Math.Min(amountFutMax, amountFut + futPozClear);
                        }
                        
                        if (sCB && sellAmountFut > 0)
                        {
                            transID++;
                            sTransId = transID;
                            sWait = 0;
                            
                            if (Form1.plaza2)
                            {
                                int opInt = 2;
                                Form1.cdTransP2.addOrder(futName, opInt, priceSellFut, sellAmountFut, Convert.ToString(threadN), transID);
                            }
                            else
                            {
                                ClassFunc.putFutOrder(transID, classFut, futName, 'S', sellAmountFut, priceSellFut, accountFut, threadN);
                            }
                            sDel = Form1.timer.ElapsedMilliseconds;//
                            sDelint = currTime;

                            sCB = false;
                            priceSellFutSave = priceSellFut;
                            sellCountSave = sellAmountFut;
                        }
                        
                    }
                }
                else if(Math.Abs(priceSellFut - priceSellFutPrev)>=sPor)
                {
                    if (sCB && sNF)
                    {
                        transID++;
                        sTransId = transID;
                        sWait = 1;
                       
                        if (Form1.plaza2)
                        {
                            Form1.cdTransP2.moveOrder(oN2, priceSellFut, transID);
                        }
                        else
                        {
                            ClassFunc.MoveOrder(classFut, futName, 0, sellAmountFut, priceBuyFut, priceSellFut, 0, oN2,
                                transID, 0);
                        }
                        sDel = Form1.timer.ElapsedMilliseconds;//
                        sDelint = currTime;

                        sCB = false;
                        priceSellFutSave = priceSellFut;
                    }
                }
                #endregion 

                //Контроль задержки возвращения коллбэка
                int delayCBbuy = 0, delayCBsell = 0;
                bool clearingTime = false;
                if (currTime >= timeClear && currTime <= timeClear+180 )
                    clearingTime = true;

                //Если коллбэк не вернулся через время, большее чем delayCBmax - к счетчикам ошибок добавляется 50 пунктов
                if (!clearingTime)
                {
                    if (!bCB || !bNF)
                    {
                        delayCBbuy = currTime - bDelint;
                        if (delayCBbuy > delayCBmax)
                        {
                            buyErr += 50;
                            bCB = true;
                        }
                    }
                    if (!sCB || !sNF)
                    {
                        delayCBsell = currTime - sDelint;
                        if (delayCBsell > delayCBmax)
                        {
                            sellErr += 50;
                            sCB = true;
                        }
                    }
                }
                
                //Запись результатов в форму для отображения
                if (currTime > saveResT)
                {
                    saveResT = currTime;
                    // Дополнение для индекса РТС - расчет финансового результата с учетом курса доллара 
                    if (baseContract == "RTS")
                    {
                        summ = (USDc / 50) * summ;
                    }
                    //___________________________________________________________________________________
                    if (summ > summax) summax = summ;
                    if (summ < summin) summin = summ;

                    //Запись в структуру результатов для данного потока параметров робота
                    Form1.res[threadN].summ = summ; // финансовый результат (прибыль/убыток)
                    Form1.res[threadN].sumMax = summax;// максимальное значение финрезультата за день
                    Form1.res[threadN].sumMin = summin;// минимальное значение финрезультата за день
                    Form1.res[threadN].futpoz = futPozClear;//текущая открытая позиция
                    Form1.res[threadN].contracts = contracts;// сумма проданных и купленных контрактов
                    Form1.res[threadN].time = timeSpread;// текущее время
                    Form1.res[threadN].delay = delaySec; // средний раундтрип
                    Form1.res[threadN].trans = transID - transIDst; //количество транзакций
                    Form1.res[threadN].trades = trades;// количество сделок
                    Form1.res[threadN].err = errors;// количество ошибок
                }
                //______________________________________________________________________
    
            }
            int n = 0;
            // Окончание торговли - закрытие позиций
            do
            {
                System.Threading.Thread.Sleep(1000);
                //Считывание открытой позиции
                ClassFunc.DBReadExec(Form1.exec, futName, ref futPozBuy, ref futPozSell, ref summTotal, ref execTableCode,
                    ref contracts,ref trades,threadN);
                futPozClear = futPozBuy - futPozSell;
                // Считывание активных ордеров
                if (Form1.plaza2)
                {
                    ClassP2.readActiveOrder(ref ordActArr, ref ordActArrPos, Convert.ToString(threadN));
                }
                else
                {
                    ClassFunc.readOrder(ref ordActArr, ref ordActArrPos, Convert.ToString(threadN));
                } 
                // Удаление активных ордеров
                if (ordActArrPos >= 0)
                {
                    for (int i = 0; i <= ordActArrPos; i++)
                    {
                        transID++;

                        if (Form1.plaza2)
                        {
                            Form1.cdTransP2.DelOrder(ordActArr[i], transID);
                        }
                        else
                        {
                            ClassFunc.killFutOrder(transID, accountFut, baseContract, 'N', futName, ordActArr[i]);
                        }
                        
                        System.Threading.Thread.Sleep(500);
                    }
                }
                //Закрытие позиции
                if (futPozClear != 0)
                {
                    transID++;
                    int AmF = 0;
                    double PrF = 0;
                    if (futPozClear < 0)
                    {
                        AmF = -futPozClear;
                        PrF = prFut[currTime] * 1.01;
                        if (baseContract == "RTS")
                        {
                            PrF = (float)Math.Round(PrF / priceStep) * priceStep;
                        }
                        transID++;
                        
                        if (Form1.plaza2)
                        {
                            Form1.cdTransP2.addOrder(futName, 1, PrF, AmF, Convert.ToString(threadN), transID);
                        }
                        else
                        {
                            ClassFunc.putFutOrder(transID, classFut, futName, 'B', AmF, PrF, accountFut, threadN);
                        }
                        
                        System.Threading.Thread.Sleep(2000);
                    }
                    else
                    {
                        AmF = futPozClear;
                        PrF = prFut[currTime] * 0.99;
                        if (baseContract == "RTS")
                        {
                            PrF = (float)Math.Round(PrF / priceStep) * priceStep;
                        }
                        transID++;
                        
                        if (Form1.plaza2)
                        {
                            Form1.cdTransP2.addOrder(futName, 2, PrF, AmF, Convert.ToString(threadN), transID);
                        }
                        else
                        {
                            ClassFunc.putFutOrder(transID, classFut, futName, 'S', AmF, PrF, accountFut, threadN);
                        }
                        
                        System.Threading.Thread.Sleep(2000);
                    }
                }
                n++;
            } while ((futPozClear != 0||ordActArrPos>=0) && n<3);

            int fpoz = 0;
            int expoz = 0;
            int pf = 0;
            double sT = 0;
            double lastPrice = 0;
            TimeSpan bgTime;
            bgTime = new TimeSpan(10, 30, 0);

            // Создание массива prof изменения финрезультата в течение дня, для последующего анализа
            double[] prof = new double[47000];
            for (int i = (int)bgTime.TotalSeconds; i <= (int)endTime.TotalSeconds; i++)
            {
                while (expoz <= Form1.exec.pos && Form1.exec.exetime[expoz].TotalSeconds <= i)
                {
                    if (Form1.exec.exename[expoz] == futName && Form1.exec.comment[expoz] == threadN &&
                        Form1.exec.exedir[expoz] == "Купля")
                    {
                        fpoz += Form1.exec.exequan[expoz];
                        sT -= Form1.exec.exequan[expoz] * Form1.exec.exeprice[expoz];
                        lastPrice = Form1.exec.exeprice[expoz];
                    }
                    else if (Form1.exec.exename[expoz] == futName && Form1.exec.comment[expoz] == threadN &&
                        Form1.exec.exedir[expoz] == "Продажа")
                    {
                        fpoz -= Form1.exec.exequan[expoz];
                        sT += Form1.exec.exequan[expoz] * Form1.exec.exeprice[expoz];
                        lastPrice = Form1.exec.exeprice[expoz];
                    }
                    expoz++;
                }
                prof[pf] = sT + fpoz * lastPrice;
                pf++;
            }

            DateTime dt=DateTime.Today;
            string id = String.Format("{0}.({1}_{2})", dt.ToString("yyMMdd"), futName, threadN);
            System.IO.StreamWriter pwrite = new System.IO.StreamWriter(@"Profit/"+id+".csv", false);
            string logRow;
            //Запись массива prof в файл
            for (int i = 0; i < pf; i++)
            {
                logRow = String.Format("{0};{1}", bgTime.Add(new TimeSpan(0,0,i)).ToString(), prof[i]);
                pwrite.WriteLine(logRow);
            }
            pwrite.Close();

            System.IO.StreamWriter iwrite = new System.IO.StreamWriter(ClassP2.rootPath + ".mdata"+threadN+".csv", false);

            //Запись некоторых массивов робота - спреда, цены, индекса в файл для анализа
            for (int i = 0; i <= currTime; i++)
            {
                logRow = String.Format("{0};{1};{2};{3};{4}", new TimeSpan(0, 0, i+timeStart).ToString(), prSt[i], prFut[i],
                    spread[i], nspread[i]);
                iwrite.WriteLine(logRow);
            }
            iwrite.Close();
          
            logRow = String.Format("{0}.{1}.{2} summ:{3} contr:{4} id:{5}", dt.Day,dt.Month,dt.Year,summ,contracts,threadN);
            ClassFunc.resWrite(logRow);
        }
        #region IDisposable Members
     
        public void Dispose()
        {
            // TODO:  Add ClassMain.Dispose implementation
            Stopped = true;
        }

        #endregion
        
        //Функция коллбэка. Хранилище данных коллбэка - структура trdata
        private void onTransactionReply(callbackTransData trdata,ref int code)
        {
            if (code < trdata.pos)
            {
                for (int i = code + 1; i <= trdata.pos; i++)
                {
                    //Проверка коллбэка ордера на покупку
                    if (trdata.transactionID[i] == bTransId)
                    {
                        if (trdata.transactionResult[i] == 0)
                        {
                            // Выставление ордера (ADD)
                            if (bWait == 0)
                            {
                                string[] strarray = trdata.transactionReplyMessage[i].Split('.', ':', ',');
                                if (strarray[0] == "Операция выполнена успешно")
                                {
                                    oN1 =Convert.ToInt64(trdata.orderNumber[i]); // получение номера ордера
                                    priceBuyFutPrev = priceBuyFutSave; 
                                    //mTB = trdata.cbtime[i].TimeOfDay.TotalMilliseconds;
                                    buyCount += buyCountSave; //количество активных ордеров на покупку
                                    buyErr = 0;
                                   
                                }
                                else
                                {
                                    buyErr++;// счетчик ошибок
                                }
                            }
                            // Передвижение ордера (MOVE)
                            else if (bWait == 1)
                            {
                                string[] strarray = trdata.transactionReplyMessage[i].Split('.', ':', ',');
                                if (strarray[0] == "Операция выполнена успешно")// && Convert.ToInt32(strarray[2]) != 0)
                                {
                                    buyErr = 0;
                                    oN1 = Convert.ToInt64(trdata.orderNumber[i]); //Convert.ToInt32(strarray[2]);
                                    priceBuyFutPrev = priceBuyFutSave;
                                    //mTB = trdata.cbtime[i].TimeOfDay.TotalMilliseconds;
                                }
                                else if (strarray[0] == "Не найдена заявка для перестановки")
                                {
                                    //Ордер исполнился или удален
                                    bNF=false; // блокировка до прихода подтверждения по сделкам 
                                    
                                }
                                else
                                {
                                    buyErr++;
                                }
                            }
                        }
                        else
                            buyErr++;
                        //Коллбэк пришел
                        bCB = true;

                        // Вычисление задержки коллбэка и занесения в массив delay для последующего вычисления среднй задержки
                        double delTime = trdata.cbtime[i] - bDel;
                        if (delTime < 0)
                            delTime = 0;

                        if (delayi <= delayCount - 1)
                        {
                            delay[delayi] = delTime / 1000;
                            delayi++;
                        }
                        else
                        {
                            delayi = 0;
                            delay[delayi] = delTime / 1000;
                            delayi++;
                        }
                        delaySec = ClassFunc.Average(delay);
                    }

                    //Проверка коллбэка на продажу - все аналогично покупке
                    if (trdata.transactionID[i] == sTransId)
                    {
                        if (trdata.transactionResult[i] == 0)
                        {
                            if (sWait == 0)
                            {
                                string[] strarray = trdata.transactionReplyMessage[i].Split('.', ':', ',');
                                if (strarray[0] == "Операция выполнена успешно")
                                {
                                    oN2 = Convert.ToInt64(trdata.orderNumber[i]);
                                    priceSellFutPrev = priceSellFutSave;
                                    //mTS = DateTime.Now.TimeOfDay.TotalMilliseconds;
                                    sellCount += sellCountSave;
                                    sellErr = 0;
                                }
                                else
                                {
                                    sellErr++;
                                }
                            }
                            else if (sWait == 1)
                            {
                                string[] strarray = trdata.transactionReplyMessage[i].Split('.', ':', ',');
                                if (strarray[0] == "Операция выполнена успешно")// && Convert.ToInt32(strarray[2]) != 0)
                                {
                                    sellErr = 0;
                                    oN2 = Convert.ToInt64(trdata.orderNumber[i]); //Convert.ToInt32(strarray[2]);
                                    priceSellFutPrev = priceSellFutSave;
                                    //mTS = trdata.cbtime[i].TimeOfDay.TotalMilliseconds;
                                }
                                else if (strarray[0] == "Не найдена заявка для перестановки")
                                {
                                    sNF = false;
                                    //mTS = trdata.cbtime[i].TimeOfDay.TotalMilliseconds;
                                }
                                else
                                {
                                    sellErr++;
                                    //mTS = trdata.cbtime[i].TimeOfDay.TotalMilliseconds;
                                }
                            }
                        }
                        else
                            sellErr++;
                        sCB = true;

                        double delTime = trdata.cbtime[i] - sDel;
                        if (delTime < 0)
                            delTime = 0;

                        if (delayi <= delayCount - 1)
                        {
                            delay[delayi] = delTime / 1000;
                            delayi++;
                        }
                        else
                        {
                            delayi = 0;
                            delay[delayi] = delTime / 1000;
                            delayi++;
                        }
                        delaySec = ClassFunc.Average(delay);
                    }
                    //ClassFunc.logWrite(trdata.transactionReplyMessage[i]);
                }
                code = trdata.pos;
            }
        }
    }


}
