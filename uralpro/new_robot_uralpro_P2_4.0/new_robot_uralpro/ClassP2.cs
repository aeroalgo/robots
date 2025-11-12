using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.IO;
using P2ClientGateMTA;
using System.Windows.Forms;
using System.Runtime.InteropServices;
using System.Data;

namespace new_robot_uralpro
{
    //Класс получения потоков репликации Плаза2
    class ClassP2
    {
        System.Windows.Forms.Timer myTimer = new System.Windows.Forms.Timer();

        public string myCode;
        public string nameGlass;
        public ClassP2(string myCodeKey)
        {
            myCode = myCodeKey;
        }
        public static bool m_stop = false;

        CP2Connection conn;
        CP2DataStream streamTrades,streamDeals;
        long revStreamTradesTableOrders_log = -1;
        long revStreamTradesTableDeal = -1;
        long revStreamTradesTableUserDeal = -1;
        long revStreamAggregates = -1;
        CP2DataStream streamPos;
        CP2DataStream streamPart;
        CP2DataStream streamFutInfo;
        CP2DataStream m_streamFutCommon;
        CP2DataStream streamAggregates;

        StreamWriter sw_logFile;
        StreamWriter sw_futInfo;
        StreamWriter sw_Aggregates;

        public static string rootPath = @"c:\log\" + DateTime.Now.ToString("yyMMdd") + ".P2";
        string pathP2Log = rootPath + ".log";
        string pathQuot = rootPath + ".quot.csv";
        string pathFutInfo = rootPath + ".futinfo.csv";
        string pathOrder = rootPath + ".order.csv";
        string pathRevOrders_log = rootPath + ".revOrders.csv";
        string pathRevDeal = rootPath + ".revDeal.log";
        string pathRevUserDeal=rootPath + ".revUserDeal.log";
        string pathRevAggregates = rootPath + ".revAggregates.csv";
        string pathGlass = rootPath + ".glass.csv";
        string pathP2Trans = rootPath + ".P2Trans.csv";
        string pathCB = rootPath + ".cb.csv";

        //Форма контроля и отображения данных Плаза2
        FormPlaza2 plaza2Form;
        public static ComDatafutInfo cdFutInfo = new ComDatafutInfo(1000);

        //Таблицы данных для формы plaza2Form
        static DataTable ordTable;
        DataTable posTable;
        DataTable partTable;
        DataTable futCommonTable;

        DateTime servTime = new DateTime();
        string streamTradesState = "";
        string streamDealsState = "";
        string streamAggregatesState = "";

        [MTAThread]
        public void Run()
        {
            System.Threading.Thread.Sleep(1000);
            plaza2Form = new FormPlaza2();
            plaza2Form.StartPosition = FormStartPosition.Manual;
            plaza2Form.Location = new System.Drawing.Point(150, 100);
            plaza2Form.Show();

            //Формирование таблиц данных для отображения в plaza2Form
            #region datatable
            DataColumn workColumn = null;

            ordTable = new DataTable("ordTable");
            ordTable.MinimumCapacity = 10;
            workColumn = ordTable.Columns.Add("id_ord", System.Type.GetType("System.Int64"));
            workColumn.AllowDBNull = false;
            workColumn.Unique = true;
            workColumn = ordTable.Columns.Add("moment", System.Type.GetType("System.String"));
            workColumn.AllowDBNull = false;
            workColumn = ordTable.Columns.Add("short_isin", System.Type.GetType("System.String"));
            workColumn.AllowDBNull = false;
            workColumn = ordTable.Columns.Add("dir", System.Type.GetType("System.String"));
            workColumn.AllowDBNull = false;
            workColumn = ordTable.Columns.Add("price", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = ordTable.Columns.Add("amount", System.Type.GetType("System.Int32"));
            workColumn.AllowDBNull = false;
            workColumn = ordTable.Columns.Add("comment", System.Type.GetType("System.String"));
            ordTable.PrimaryKey = new DataColumn[] { ordTable.Columns["id_ord"] };

            BindingSource bsOrdTable = new BindingSource();
            bsOrdTable.DataSource = ordTable;
            //plaza2Form.dataGridView1.DataSource = bsOrdTable;
            plaza2Form.addBS1(bsOrdTable);

            posTable = new DataTable("posTable");
            ordTable.MinimumCapacity = 2;
            workColumn = posTable.Columns.Add("short_isin", System.Type.GetType("System.String"));
            workColumn.AllowDBNull = false;
            workColumn.Unique = true;
            workColumn = posTable.Columns.Add("open_qty", System.Type.GetType("System.Int32"));
            workColumn.AllowDBNull = false;
            workColumn = posTable.Columns.Add("buys_qty", System.Type.GetType("System.Int32"));
            workColumn.AllowDBNull = false;
            workColumn = posTable.Columns.Add("sells_qty", System.Type.GetType("System.Int32"));
            workColumn.AllowDBNull = false;
            workColumn = posTable.Columns.Add("pos", System.Type.GetType("System.Int32"));
            workColumn.AllowDBNull = false;
            posTable.PrimaryKey = new DataColumn[] { posTable.Columns["short_isin"] };

            BindingSource bsPosTable = new BindingSource();
            bsPosTable.DataSource = posTable;
            //plaza2Form.dataGridView2.DataSource = bsPosTable;
            plaza2Form.addBS2(bsPosTable);

            partTable = new DataTable("partTable");
            partTable.MinimumCapacity = 2;
            workColumn = partTable.Columns.Add("client_code", System.Type.GetType("System.String"));
            workColumn.AllowDBNull = false;
            workColumn.Unique = true;
            workColumn = partTable.Columns.Add("money_old", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = partTable.Columns.Add("money_amount", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = partTable.Columns.Add("money_free", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = partTable.Columns.Add("money_blocked", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = partTable.Columns.Add("fee", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            partTable.PrimaryKey = new DataColumn[] { partTable.Columns["client_code"] };

            BindingSource bsPartTable = new BindingSource();
            bsPartTable.DataSource = partTable;
            //plaza2Form.dataGridView3.DataSource = bsPartTable;
            plaza2Form.addBS3(bsPartTable);

            futCommonTable = new DataTable("futCommonTable");
            futCommonTable.MinimumCapacity = 3;
            workColumn = futCommonTable.Columns.Add("short_isin", System.Type.GetType("System.String"));
            workColumn.AllowDBNull = false;
            workColumn.Unique = true;
            workColumn = futCommonTable.Columns.Add("price", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = futCommonTable.Columns.Add("min_price", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = futCommonTable.Columns.Add("max_price", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            workColumn = futCommonTable.Columns.Add("deal_time", System.Type.GetType("System.TimeSpan"));
            workColumn.AllowDBNull = false;
            workColumn = futCommonTable.Columns.Add("capital", System.Type.GetType("System.Double"));
            workColumn.AllowDBNull = false;
            futCommonTable.PrimaryKey = new DataColumn[] { futCommonTable.Columns["short_isin"] };

            BindingSource bsFutCommonTable = new BindingSource();
            bsFutCommonTable.DataSource = futCommonTable;
            //plaza2Form.dataGridView4.DataSource = bsFutCommonTable;
            plaza2Form.addBS4(bsFutCommonTable);

            #endregion datatable

            myTimer.Tick += new EventHandler(myTimerEvent);
            myTimer.Interval = 1000;
            myTimer.Start();

            try
            {
                // Объект "соединение" и параметры соединения с приложением P2MQRouter
                conn = new CP2ConnectionClass();
                conn.Host = "localhost";
                conn.Port = 4001;
                conn.AppName = "p2_f";

                IP2ConnectionEvent_ConnectionStatusChangedEventHandler connStatusHandler = new IP2ConnectionEvent_ConnectionStatusChangedEventHandler(ConnectionStatusChanged);
                conn.ConnectionStatusChanged += connStatusHandler;

                cdFutInfo.readFromFile("futinfo.csv");
                cdFutInfo.createHash();

                //Подписка на потоки Плаза2
                #region stream
                //Поток получения данных по фьючерсам
                streamFutInfo = new CP2DataStream();
                streamFutInfo.DBConnString = "";
                streamFutInfo.type = TRequestType.RT_COMBINED_DYNAMIC;
                streamFutInfo.StreamName = "FORTS_FUTINFO_REPL";
                streamFutInfo.TableSet = new CP2TableSetClass();
                streamFutInfo.TableSet.InitFromIni2("sheme.ini", "fut_info_Scheme");

                IP2DataStreamEvents_StreamStateChangedEventHandler StateHandler = new IP2DataStreamEvents_StreamStateChangedEventHandler(StreamStateChanged);
                streamFutInfo.StreamStateChanged += StateHandler;

                IP2DataStreamEvents_StreamDataInsertedEventHandler InsHandlerFutInfo = new IP2DataStreamEvents_StreamDataInsertedEventHandler(StreamDataInsertedFutInfo);
                streamFutInfo.StreamDataInserted += InsHandlerFutInfo;

                IP2DataStreamEvents_StreamDataDeletedEventHandler DelHandler = new IP2DataStreamEvents_StreamDataDeletedEventHandler(StreamDataDeleted);
                streamFutInfo.StreamDataDeleted += DelHandler;

                IP2DataStreamEvents_StreamDatumDeletedEventHandler DatumDeleted = new IP2DataStreamEvents_StreamDatumDeletedEventHandler(StreamDataDatumDeleted);
                streamFutInfo.StreamDatumDeleted += DatumDeleted;

                IP2DataStreamEvents_StreamLifeNumChangedEventHandler LifeNumHandler = new IP2DataStreamEvents_StreamLifeNumChangedEventHandler(StreamLifeNumChanged);
                streamFutInfo.StreamLifeNumChanged += LifeNumHandler;

              //Поток получения данных стакана
                //_________fut_aggr_stream_____________________________________________________________________
                if (Form1.glassP2)
                {
                    streamAggregates = new CP2DataStream();
                    streamAggregates.DBConnString = "";
                    streamAggregates.type = TRequestType.RT_COMBINED_DYNAMIC;
                    streamAggregates.StreamName = "FORTS_FUTAGGR20_REPL";
                    streamAggregates.TableSet = new CP2TableSetClass();
                    streamAggregates.TableSet.InitFromIni2("sheme.ini", "CustReplScheme");

                    streamAggregates.StreamStateChanged += StateHandler;

                    IP2DataStreamEvents_StreamDataInsertedEventHandler InsHandlerAggregates = new IP2DataStreamEvents_StreamDataInsertedEventHandler(StreamDataInsertedAggr);
                    streamAggregates.StreamDataInserted += InsHandlerAggregates;

                    streamAggregates.StreamDataDeleted += DelHandler;

                    streamAggregates.StreamDatumDeleted += DatumDeleted;

                    streamAggregates.StreamLifeNumChanged += LifeNumHandler;

                    IP2DataStreamEvents_StreamDataBeginEventHandler BeginHandlerFutAggr20 = new IP2DataStreamEvents_StreamDataBeginEventHandler(StreamDataBegin);

                    streamAggregates.StreamDataBegin += BeginHandlerFutAggr20;

                    IP2DataStreamEvents_StreamDataEndEventHandler EndHandlerFutAggr20 = new IP2DataStreamEvents_StreamDataEndEventHandler(StreamDataEnd);

                    streamAggregates.StreamDataEnd += EndHandlerFutAggr20;

                    readRev(pathRevAggregates, ref revStreamAggregates);
                }
                //_____________________________________________________________________________________________
                
                //Поток получения всех сделок
                Form1.currDataFut.readData(pathQuot);//Считывание данных из файла - при восстановлении
                Form1.exec.readData(Form1.currDataFut);
                readRev(pathRevDeal, ref revStreamTradesTableDeal);
                readTable(pathOrder, ref ordTable);
                readRev(pathRevOrders_log, ref revStreamTradesTableOrders_log);

                streamTrades = new CP2DataStream();
                streamTrades.DBConnString = "";
                streamTrades.type = TRequestType.RT_COMBINED_DYNAMIC;
                streamTrades.StreamName = "FORTS_FUTTRADE_REPL";
                streamTrades.TableSet = new CP2TableSetClass();
                streamTrades.TableSet.InitFromIni2("sheme.ini", "fut_trades_Scheme");

                streamTrades.StreamStateChanged += StateHandler;

                IP2DataStreamEvents_StreamDataInsertedEventHandler InsHandler = new IP2DataStreamEvents_StreamDataInsertedEventHandler(StreamDataInserted);
                streamTrades.StreamDataInserted += InsHandler;

                streamTrades.StreamDataDeleted += DelHandler;

                streamTrades.StreamDatumDeleted += DatumDeleted;

                streamTrades.StreamLifeNumChanged += LifeNumHandler;

                streamDeals = new CP2DataStream();
                streamDeals.DBConnString = "";
                streamDeals.type = TRequestType.RT_COMBINED_DYNAMIC;
                streamDeals.StreamName = "FORTS_DEALS_REPL";
                streamDeals.TableSet = new CP2TableSet();
                streamDeals.TableSet.InitFromIni2("sheme.ini", "fut_deals_Scheme");

                streamDeals.StreamStateChanged += StateHandler;

                IP2DataStreamEvents_StreamDataInsertedEventHandler InsHandlerUser = new IP2DataStreamEvents_StreamDataInsertedEventHandler(StreamDataDealInserted);
                streamDeals.StreamDataInserted += InsHandlerUser;

                streamDeals.StreamDataDeleted += DelHandler;

                streamDeals.StreamDatumDeleted += DatumDeleted;

                streamDeals.StreamLifeNumChanged += LifeNumHandler;

                //Поток получения данных по позициям трейдера
                streamPos = new CP2DataStream();
                streamPos.DBConnString = "";
                streamPos.type = TRequestType.RT_REMOTE_ONLINE;
                streamPos.StreamName = "FORTS_POS_REPL";
                streamPos.TableSet = new CP2TableSetClass();
                streamPos.TableSet.InitFromIni2("sheme.ini", "pos_Scheme");

                streamPos.StreamStateChanged += StateHandler;

                IP2DataStreamEvents_StreamDataInsertedEventHandler InsHandlerPos = new IP2DataStreamEvents_StreamDataInsertedEventHandler(StreamDataInsertedPos);
                streamPos.StreamDataInserted += InsHandlerPos;

                streamPos.StreamDataDeleted += DelHandler;

                streamPos.StreamLifeNumChanged += LifeNumHandler;

                //Поток получения данных аккаунта трейдера
                streamPart = new CP2DataStream();
                streamPart.DBConnString = "";
                streamPart.type = TRequestType.RT_REMOTE_ONLINE;
                streamPart.StreamName = "FORTS_PART_REPL";
                streamPart.TableSet = new CP2TableSetClass();
                streamPart.TableSet.InitFromIni2("sheme.ini", "part_Scheme");

                streamPart.StreamStateChanged += StateHandler;

                IP2DataStreamEvents_StreamDataInsertedEventHandler InsHandlerPart = new IP2DataStreamEvents_StreamDataInsertedEventHandler(StreamDataInsertedPart);
                streamPart.StreamDataInserted += InsHandlerPart;

                streamPart.StreamDataDeleted += DelHandler;

                streamPart.StreamLifeNumChanged += LifeNumHandler;

                ////////////////////////////
                //m_streamFutCommon = new CP2DataStream();
                //m_streamFutCommon.DBConnString = "";
                //m_streamFutCommon.type = TRequestType.RT_REMOTE_ONLINE;
                //m_streamFutCommon.StreamName = "FORTS_FUTCOMMON_REPL";
                //m_streamFutCommon.TableSet = new CP2TableSetClass();
                //m_streamFutCommon.TableSet.InitFromIni2("sheme.ini", "fut_common_Scheme");

                //m_streamFutCommon.StreamStateChanged += StateHandler;

                //IP2DataStreamEvents_StreamDataInsertedEventHandler InsHandlerFutCommon = new IP2DataStreamEvents_StreamDataInsertedEventHandler(StreamDataInsertedFutCommon);
                //m_streamFutCommon.StreamDataInserted += InsHandlerFutCommon;

                //m_streamFutCommon.StreamLifeNumChanged += LifeNumHandler;

                //m_streamFutCommon.StreamDataDeleted += DelHandler;
                #endregion stream
            }
            catch (Exception e)
            {
                int hRes = Marshal.GetHRForException(e);
                Console.WriteLine("Exception {0}", e.Message);
                MessageBox.Show("Ошибка: " + e.Message, "Ошибка");
                if (hRes == -2147205116) // P2ERR_INI_FILE_NOT_FOUND
                {
                    string s = "Can't find one or more ini file";
                    MessageBox.Show("Ошибка: " + s, "Ошибка");
                }
            }
            while (!m_stop)
            {
                try
                {
                    //System.Threading.Thread.Sleep(0);
                    // создаем соединение с роутером
                    if (conn.Status == 1)
                    {
                        conn.Connect();
                        //conn.Connect2("APPNAME=p2_f;LRPCQ_PORT=4001");
                        System.Threading.Thread.Sleep(2000);
                    }
                    try
                    {
                        while (!m_stop)
                        {
                            System.Threading.Thread.Sleep(0);
                            Application.DoEvents();
                            try
                            {
                                #region tradesOpen
                                if (streamTrades.State == TDataStreamState.DS_STATE_ERROR ||
                                    streamTrades.State == TDataStreamState.DS_STATE_CLOSE)
                                {
                                    if (streamTrades.State == TDataStreamState.DS_STATE_ERROR)
                                    {
                                        streamTrades.Close();
                                        System.Threading.Thread.Sleep(500);
                                    }
                                    streamTrades.TableSet.set_rev("user_deal", revStreamTradesTableUserDeal + 1);
                                    streamTrades.TableSet.set_rev("deal", revStreamTradesTableDeal + 1);
                                    streamTrades.TableSet.set_rev("orders_log", revStreamTradesTableOrders_log + 1);
                                    streamTrades.Open(conn);
                                }

                                if (streamDeals.State == TDataStreamState.DS_STATE_ERROR ||
                                    streamDeals.State == TDataStreamState.DS_STATE_CLOSE)
                                {
                                    if (streamDeals.State == TDataStreamState.DS_STATE_ERROR)
                                    {
                                        streamDeals.Close();
                                        System.Threading.Thread.Sleep(500);
                                    }
                                    streamDeals.TableSet.set_rev("deal", revStreamTradesTableDeal + 1);
                                    streamDeals.Open(conn);
                                }
                                //___Glass__________________________________________________________________________
                                if (Form1.glassP2)
                                {
                                    if (streamAggregates.State == TDataStreamState.DS_STATE_ERROR ||
                                        streamAggregates.State == TDataStreamState.DS_STATE_CLOSE)
                                    {
                                        if (streamAggregates.State == TDataStreamState.DS_STATE_ERROR)
                                        {
                                            streamAggregates.Close();
                                        }
                                        streamAggregates.TableSet.set_rev("orders_aggr", revStreamAggregates + 1);
                                        streamAggregates.Open(conn);
                                    }
                                }
                                //_____________________________________________________________________________________

                                if (streamPos.State == TDataStreamState.DS_STATE_ERROR ||
                                    streamPos.State == TDataStreamState.DS_STATE_CLOSE)
                                {
                                    if (streamPos.State == TDataStreamState.DS_STATE_ERROR)
                                    {
                                        streamPos.Close();
                                    }
                                    streamPos.Open(conn);
                                }
                                if (streamPart.State == TDataStreamState.DS_STATE_ERROR ||
                                      streamPart.State == TDataStreamState.DS_STATE_CLOSE)
                                {
                                    if (streamPart.State == TDataStreamState.DS_STATE_ERROR)
                                    {
                                        streamPart.Close();
                                    }
                                    streamPart.Open(conn);
                                }
                                if (streamFutInfo.State == TDataStreamState.DS_STATE_ERROR ||
                                        streamFutInfo.State == TDataStreamState.DS_STATE_CLOSE)
                                {
                                    if (streamFutInfo.State == TDataStreamState.DS_STATE_ERROR)
                                    {
                                        streamFutInfo.Close();
                                    }
                                    streamFutInfo.Open(conn);
                                }
                                /////////////////////
                                //if (m_streamFutCommon.State == TDataStreamState.DS_STATE_ERROR ||
                                //        m_streamFutCommon.State == TDataStreamState.DS_STATE_CLOSE)
                                //{
                                //    if (m_streamFutCommon.State == TDataStreamState.DS_STATE_ERROR)
                                //    {
                                //        m_streamFutCommon.Close();
                                //    }
                                //    m_streamFutCommon.Open(conn);
                                //}
                                #endregion tradesOpen
                            }
                            catch (System.Runtime.InteropServices.COMException e)
                            {
                                LogWriteLine(e.Message + "   " + e.ErrorCode);
                            }
                            uint cookie;
                            // обрабатываем пришедшее сообщение. Обработка идет в интерфейсах обратного вызова
                            conn.ProcessMessage(out cookie, 500);
                        }
                    }
                    catch (System.Runtime.InteropServices.COMException e)
                    {
                        LogWriteLine(e.Message + "   " + e.ErrorCode);
                    }
                    #region tradesClose
                    if (streamTrades.State != TDataStreamState.DS_STATE_CLOSE)
                        try
                        {
                            streamTrades.Close();
                        }
                        catch (System.Runtime.InteropServices.COMException e)
                        {
                            LogWriteLine(e.Message + "   " + e.ErrorCode);
                        }
                    if (streamDeals.State != TDataStreamState.DS_STATE_CLOSE)
                        try
                        {
                            streamDeals.Close();
                        }
                        catch (System.Runtime.InteropServices.COMException e)
                        {
                            LogWriteLine(e.Message + "   " + e.ErrorCode);
                        }

                    if ( Form1.glassP2 && streamAggregates.State != TDataStreamState.DS_STATE_CLOSE)
                        try
                        {
                            streamAggregates.Close();
                        }
                        catch (System.Runtime.InteropServices.COMException e)
                        {
                            LogWriteLine(e.Message + "   " + e.ErrorCode);
                        }

                    if (streamPos.State != TDataStreamState.DS_STATE_CLOSE)
                        try
                        {
                            streamPos.Close();
                        }
                        catch (System.Runtime.InteropServices.COMException e)
                        {
                            LogWriteLine(e.Message + "   " + e.ErrorCode);
                        }
                    if (streamPart.State != TDataStreamState.DS_STATE_CLOSE)
                        try
                        {
                            streamPart.Close();
                        }
                        catch (System.Runtime.InteropServices.COMException e)
                        {
                            LogWriteLine(e.Message + "   " + e.ErrorCode);
                        }
                    if (streamFutInfo.State != TDataStreamState.DS_STATE_CLOSE)
                        try
                        {
                            streamFutInfo.Close();
                        }
                        catch (System.Runtime.InteropServices.COMException e)
                        {
                            LogWriteLine(e.Message + "   " + e.ErrorCode);
                        }
                    ////////////////////
                    //if (m_streamFutCommon.State != TDataStreamState.DS_STATE_CLOSE)
                    //    try
                    //    {
                    //        m_streamFutCommon.Close();
                    //    }
                    //    catch (System.Runtime.InteropServices.COMException e)
                    //    {
                    //        LogWriteLine(e.Message + "   " + e.ErrorCode);
                    //    }
                    #endregion tradesClose
                    conn.Disconnect();
                }
                catch (System.Runtime.InteropServices.COMException e)
                {
                    LogWriteLine(e.Message + "   " + e.ErrorCode);
                }
                catch (System.Exception e)
                {
                    LogWriteLine(e.Message + "   " + e.Source);
                }
            }
            
            Form1.saveData(pathQuot); // Запись данных по всем сделкам при окончании работы Плаза2
            saveRev(pathRevDeal, revStreamTradesTableDeal);
            saveTable(pathOrder, ordTable); //Запись данных по ордерам в рынке
            saveRev(pathRevOrders_log, revStreamTradesTableOrders_log);
            saveRev(pathRevAggregates, revStreamAggregates);
            saveRev(pathRevUserDeal, revStreamTradesTableUserDeal);
            Form1.cdTransP2.saveData(pathP2Trans); //Запись данных по транзакциям
            Form1.cbTrD.saveData(pathCB); //Запись данных коллбэка
        }

        [MTAThread]
        void ConnectionStatusChanged(CP2Connection conn, TConnectionStatus newStatus)
        {
            String state = "MQ connection state ";
            if ((newStatus & TConnectionStatus.CS_CONNECTION_BUSY) != 0)
            {
                state += "BUSY";
            }
            else if ((newStatus & TConnectionStatus.CS_CONNECTION_CONNECTED) != 0)
            {
                state += "CONNECTED";
            }
            else if ((newStatus & TConnectionStatus.CS_CONNECTION_DISCONNECTED) != 0)
            {
                state += "DISCONNECTED";
            }
            else if ((newStatus & TConnectionStatus.CS_CONNECTION_INVALID) != 0)
            {
                state += "INVALID";
            }

            state += " router state ";
            if ((newStatus & TConnectionStatus.CS_ROUTER_CONNECTED) != 0)
            {
                // Когда соединились - запрашиваем адрес сервера-обработчика
                state += "CONNECTED";
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_DISCONNECTED) != 0)
            {
                state += "DISCONNECTED";
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_LOGINFAILED) != 0)
            {
                state += "LOGINFAILED";
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_NOCONNECT) != 0)
            {
                state += "NOCONNECT";
            }
            else if ((newStatus & TConnectionStatus.CS_ROUTER_RECONNECTING) != 0)
            {
                state += "RECONNECTING";
            }
            LogWriteLine(state);
        }
        [MTAThread]
        void StreamStateChanged(CP2DataStream stream, TDataStreamState newState)
        {

            String state = "Stream " + stream.StreamName + " state: ";
            switch (newState)
            {
                case TDataStreamState.DS_STATE_CLOSE:
                    state += "CLOSE";
                    //m_opened = false;
                    break;
                case TDataStreamState.DS_STATE_CLOSE_COMPLETE:
                    state += "CLOSE_COMPLETE";
                    break;
                case TDataStreamState.DS_STATE_ERROR:
                    state += "ERROR";
                    //m_opened = false;
                    break;
                case TDataStreamState.DS_STATE_LOCAL_SNAPSHOT:
                    state += "LOCAL_SNAPSHOT";
                    break;
                case TDataStreamState.DS_STATE_ONLINE:
                    state += "ONLINE";
                    break;
                case TDataStreamState.DS_STATE_REMOTE_SNAPSHOT:
                    state += "REMOTE_SNAPSHOT";
                    break;
                case TDataStreamState.DS_STATE_REOPEN:
                    state += "REOPEN";
                    break;
            }
            if (stream.StreamName == "FORTS_FUTTRADE_REPL") streamTradesState = state;
            if (stream.StreamName == "FORTS_DEALS_REPL") streamDealsState = state;
            if (stream.StreamName == "FORTS_FUTAGGR20_REPL") streamAggregatesState = state;
            LogWriteLine(state);
        }
        [MTAThread]
        void StreamDataInserted(CP2DataStream stream, String tableName, CP2Record rec)
        {
            //Получение данных по своим сделкам
            if (tableName == "user_deal")
            {
                try
                {
                    servTime = Convert.ToDateTime(rec.GetValAsString("moment"));
                    string stname = (string)cdFutInfo.convertIsinIdToShortIsin((int)rec.GetValAsLong("isin_id"));
                    if (stname != null)
                    {
                        long rev = (long)rec.GetValAsVariant("replRev");
                        long nosystem = rec.GetValAsLong("nosystem");
                        double price = Convert.ToDouble(rec.GetValAsVariant("price"));
                        double amount = Convert.ToDouble(rec.GetValAsVariant("amount"));
                        long num = (long)rec.GetValAsVariant("id_deal");
                        long numOrder = 0;
                        int comment = 0;
                        string operationInt = "";
                        bool myDeal = false;
                        if (rec.GetValAsString("code_buy") == myCode)
                        {
                            numOrder = (long)rec.GetValAsVariant("id_ord_buy");
                            string comstr = rec.GetValAsString("comment_buy");
                            if (comstr != "")
                                comment = Convert.ToInt32(comstr);
                            operationInt = "Купля";//0
                            myDeal = true;
                        }
                        else if (rec.GetValAsString("code_sell") == myCode)
                        {
                            numOrder = (long)rec.GetValAsVariant("id_ord_sell");
                            string comstr = rec.GetValAsString("comment_sell");
                            if (comstr != "")
                                comment = Convert.ToInt32(comstr);
                            operationInt = "Продажа";//1
                            myDeal = true;
                        }

                        if (rev > revStreamTradesTableUserDeal)
                        {
                            if (servTime.Date == Form1.dt.Date && nosystem == 0)
                            {
                                //Выделение своих сделок
                                if (myDeal)
                                {
                                    Form1.exec.insertData(stname, operationInt, price, amount,
                                        servTime.TimeOfDay, num, numOrder, comment);
                                }
                            }
                            revStreamTradesTableUserDeal = rev;
                        }
                    }
                }
                catch (System.Exception e)
                {
                    LogWriteLine("event: StreamDataInserted(user_deal)   " + e.Message);
                }
            }
            else if (tableName == "orders_log")
            {
                //Получение данных по своим открытым ордерам
                try
                {
                    if (rec.GetValAsString("client_code") == myCode)
                    {
                        //LogWriteRec(rec);
                        long rev = (long)rec.GetValAsVariant("replRev");

                        if (rev > revStreamTradesTableOrders_log)
                        {
                            long ordNum = (long)rec.GetValAsVariant("id_ord");
                            int action = rec.GetValAsLong("action");
                            int amount = rec.GetValAsLong("amount");
                            string moment = Convert.ToDateTime(rec.GetValAsString("moment")).ToString("HH:mm:ss");
                            string short_isin = (string)cdFutInfo.convertIsinIdToShortIsin((int)rec.GetValAsLong("isin_id"));
                            string dir = null;
                            if (rec.GetValAsLong("dir") == 1) dir = "Купля"; else dir = "Продажа";
                            double price = Convert.ToDouble(rec.GetValAsVariant("price"));
                            string comment = rec.GetValAsString("comment");
                            DataRow tempRow;

                            if (action == 1)
                            {
                                tempRow = ordTable.NewRow();
                                tempRow[0] = ordNum;
                                tempRow[1] = moment;
                                tempRow[2] = short_isin;
                                tempRow[3] = dir;
                                tempRow[4] = price;
                                tempRow[5] = amount;
                                tempRow[6] = comment;
                                ordTable.Rows.Add(tempRow);
                            }
                            else if (action == 0)
                            {
                                ordTable.Rows.Find(ordNum).Delete();
                            }
                            else if (action == 2)
                            {
                                tempRow = ordTable.Rows.Find(ordNum);
                                int nowAmount = (int)tempRow["amount"];
                                int newAmount = nowAmount - amount;

                                if (newAmount == 0)
                                {
                                    tempRow.Delete();
                                }
                                else
                                {
                                    tempRow[0] = ordNum;
                                    tempRow[1] = moment;
                                    tempRow[2] = short_isin;
                                    tempRow[3] = dir;
                                    tempRow[4] = price;
                                    tempRow[5] = newAmount;
                                    tempRow[6] = comment;
                                    //ordTable.AcceptChanges();
                                }

                            }
                            revStreamTradesTableOrders_log = rev;
                        }
                    }
                }
                catch (System.Exception e)
                {
                    LogWriteLine("event: StreamDataInserted(order)   " + e.Message);
                    LogWriteRec(rec);
                }
            }
        }
        [MTAThread]
        void StreamDataDealInserted(CP2DataStream stream, String tableName, CP2Record rec)
        {
            if (tableName == "deal")
            {
                //Получение данных по всем сделкам
                try
                {
                    servTime = Convert.ToDateTime(rec.GetValAsString("moment"));
                    string stname = (string)cdFutInfo.convertIsinIdToShortIsin((int)rec.GetValAsLong("isin_id"));
                    if (stname != null)
                    {
                        long rev = (long)rec.GetValAsVariant("replRev");
                        long nosystem = rec.GetValAsLong("nosystem");
                        double price = Convert.ToDouble(rec.GetValAsVariant("price"));
                        double amount = Convert.ToDouble(rec.GetValAsVariant("amount"));
                        long num = (long)rec.GetValAsVariant("id_deal");
                        long numOrder = 0;
                        int comment = 0;
                        string operationInt = "";
                        bool myDeal = false;

                        if (rev > revStreamTradesTableDeal)
                        {
                            if (servTime.Date == Form1.dt.Date && nosystem == 0)
                            {
                                string operation;
                                if ((long)rec.GetValAsVariant("id_ord_sell") > (long)rec.GetValAsVariant("id_ord_buy"))
                                    operation = "Продажа";
                                else
                                    operation = "Купля";

                                Form1.currDataFut.insertData(
                                    rev,
                                    stname,
                                    operation,
                                    price,
                                    amount,
                                    servTime.TimeOfDay, num, numOrder, operationInt, comment);
                            }
                            revStreamTradesTableDeal = rev;
                        }
                    }
                }
                catch (System.Exception e)
                {
                    LogWriteLine("event: StreamDataDealInserted(deal)   " + e.Message);
                }
            }
        }
        [MTAThread]
        void StreamDataInsertedAggr(CP2DataStream stream, String tableName, CP2Record rec)
        {
            //Получение данных стакана
            try
            {
                string stname = (string)cdFutInfo.convertIsinIdToShortIsin((int)rec.GetValAsLong("isin_id"));
                long rev = (long)rec.GetValAsVariant("replRev");
                long replID = (long)rec.GetValAsVariant("replID");
                long replAct = (long)rec.GetValAsVariant("replAct");
                double price = Convert.ToDouble(rec.GetValAsVariant("price"));
                double amount = Convert.ToDouble(rec.GetValAsVariant("volume"));
                DateTime glassTime = Convert.ToDateTime(rec.GetValAsString("moment"));
                int dir = rec.GetValAsLong("dir");

                //Формирование стакана
                #region glass
                bool mtch = false;
                for (int i = 0; i < 20; i++)
                {
                    if (replID == Form1.GL.replIDbid[i])
                    {
                        if (stname == nameGlass && dir == 1 && price != 0
                            && amount != 0)
                        {
                            Form1.GL.bid[i] = price;
                            Form1.GL.volBid[i] = amount;
                            if (glassTime.TimeOfDay >= Form1.GL.glassTime && glassTime.Date == DateTime.Now.Date)
                                Form1.GL.glassTime = glassTime.TimeOfDay;
                            mtch = true;
                        }
                        else
                        {
                            Form1.GL.bid[i] = 0;
                            Form1.GL.volBid[i] = 0;
                            Form1.GL.replIDbid[i] = 0;
                        }
                        break;

                    }
                    if (replID == Form1.GL.replIDask[i])
                    {
                        if (stname == nameGlass && dir == 2 && price != 0
                            && amount != 0)
                        {
                            Form1.GL.ask[i] = price;
                            Form1.GL.volAsk[i] = amount;
                            if (glassTime.TimeOfDay >= Form1.GL.glassTime && glassTime.Date == DateTime.Now.Date)
                                Form1.GL.glassTime = glassTime.TimeOfDay;
                            mtch = true;
                        }
                        else
                        {
                            Form1.GL.ask[i] = 0;
                            Form1.GL.volAsk[i] = 0;
                            Form1.GL.replIDask[i] = 0;
                        }
                        break;

                    }
                }
                if (!mtch && stname == nameGlass && price != 0
                            && amount != 0)
                {

                    for (int i = 0; i < 20; i++)
                    {
                        if ( Form1.GL.replIDbid[i] == 0 && dir == 1)
                        {
                            Form1.GL.bid[i] = price;
                            Form1.GL.volBid[i] = amount;
                            Form1.GL.glassTime = glassTime.TimeOfDay;
                            Form1.GL.replIDbid[i] = replID;
                            mtch = true;
                            break;
                        }
                        if (Form1.GL.replIDask[i] == 0 && dir == 2)
                        {
                            Form1.GL.ask[i] = price;
                            Form1.GL.volAsk[i] = amount;
                            Form1.GL.glassTime = glassTime.TimeOfDay;
                            Form1.GL.replIDask[i] = replID;
                            mtch = true;
                            break;
                        }
                    }

                }
                if (mtch)
                {
                    for (int i = 19; i >= 0; i--)
                    {
                        for (int n = 1; n <= i; n++)
                        {
                            if (Form1.GL.bid[n - 1] < Form1.GL.bid[n])
                            {
                                double temp =  Form1.GL.bid[n - 1];
                                Form1.GL.bid[n - 1] = Form1.GL.bid[n];
                                Form1.GL.bid[n] = temp;
                                long temp1 = Form1.GL.replIDbid[n - 1];
                                Form1.GL.replIDbid[n - 1] = Form1.GL.replIDbid[n];
                                Form1.GL.replIDbid[n] = temp1;
                                double temp2 = Form1.GL.volBid[n - 1];
                                Form1.GL.volBid[n - 1] = Form1.GL.volBid[n];
                                Form1.GL.volBid[n] = temp2;
                            }
                            if (Form1.GL.ask[n - 1] > Form1.GL.ask[n] && Form1.GL.ask[n] != 0)
                            {
                                double temp = Form1.GL.ask[n - 1];
                                Form1.GL.ask[n - 1] = Form1.GL.ask[n];
                                Form1.GL.ask[n] = temp;
                                long temp1 = Form1.GL.replIDask[n - 1];
                                Form1.GL.replIDask[n - 1] = Form1.GL.replIDask[n];
                                Form1.GL.replIDask[n] = temp1;
                                double temp2 = Form1.GL.volAsk[n - 1];
                                Form1.GL.volAsk[n - 1] = Form1.GL.volAsk[n];
                                Form1.GL.volAsk[n] = temp2;
                            }
                        }
                    }
                }
                #endregion glass

                revStreamAggregates = rev;

                string s = String.Format("{0};{1};{2};{3};{4};{5}", replID,stname, price, (int)dir, amount, glassTime.TimeOfDay.ToString());
                LogWriteGlass(s);
            }
            catch (System.Exception e)
            {
                LogWriteLine("event: StreamDataInsertedAggr   " + e.Message);
            }
        }
        [MTAThread]
        void StreamDataBegin(CP2DataStream stream)
        {
            string s = String.Format("{0};{1};{2};{3};{4};{5}","Begin","","","","","");
            LogWriteGlass(s);
        }
        [MTAThread]
        void StreamDataEnd(CP2DataStream stream)
        {
            string s = String.Format("{0};{1};{2};{3};{4};{5}", "End", "", "", "", "", "");
            LogWriteGlass(s);
        }
        [MTAThread]
        void StreamDataInsertedPos(CP2DataStream stream, String tableName, CP2Record rec)
        {
            //Получение данных о позициях трейдера
            try
            {
                long isin_id = rec.GetValAsLong("isin_id");
                string short_isin = (string)cdFutInfo.convertIsinIdToShortIsin((int)isin_id);
                bool newRow = false;

                DataRow tempRow = posTable.Rows.Find((object)short_isin);
                if (tempRow == null)
                {
                    newRow = true;
                    tempRow = posTable.NewRow();
                }

                tempRow[0] = short_isin;
                tempRow[1] = rec.GetValAsLong("open_qty");
                tempRow[2] = rec.GetValAsLong("buys_qty");
                tempRow[3] = rec.GetValAsLong("sells_qty");
                tempRow[4] = rec.GetValAsLong("pos");

                if (newRow) posTable.Rows.Add(tempRow);

                //posTable.AcceptChanges();
            }
            catch (System.Exception e)
            {
                LogWriteLine("event: StreamDataInsertedPos   " + e.Message);
            }
        }
        [MTAThread]
        void StreamDataInsertedPart(CP2DataStream stream, String tableName, CP2Record rec)
        {
            //Получение данных о состоянии аккаунта
            try
            {
                string client_code = rec.GetValAsString("client_code");
                if (client_code == myCode)
                {
                    bool newRow = false;
                    DataRow tempRow = partTable.Rows.Find(client_code);

                    if (tempRow == null)
                    {
                        newRow = true;
                        tempRow = partTable.NewRow();
                    }

                    tempRow[0] = client_code;
                    //Convert.ToDouble(rec.GetValAsVariant("money_old"));
                    tempRow[1] = Convert.ToDouble(rec.GetValAsVariant("money_old"));
                    tempRow[2] = Convert.ToDouble(rec.GetValAsVariant("money_amount"));
                    tempRow[3] = Convert.ToDouble(rec.GetValAsVariant("money_free"));
                    tempRow[4] = Convert.ToDouble(rec.GetValAsVariant("money_blocked"));
                    tempRow[5] = Convert.ToDouble(rec.GetValAsVariant("fee"));

                    if (newRow) partTable.Rows.Add(tempRow);

                    //partTable.AcceptChanges();
                }
            }
            catch (System.Exception e)
            {
                LogWriteLine("event: StreamDataInsertedPart   " + e.Message);
            }
        }
        [MTAThread]
        void StreamDataInsertedFutInfo(CP2DataStream stream, String tableName, CP2Record rec)
        {
            try
            {
                if (sw_futInfo == null)
                {
                    sw_futInfo = new StreamWriter(pathFutInfo, false, System.Text.Encoding.Unicode);
                }
                uint count = rec.Count;
                for (uint i = 0; i < count; ++i)
                {
                    if (i != count - 1)
                    {
                        sw_futInfo.Write(rec.GetValAsStringByIndex(i) + ";");
                    }
                    else
                    {
                        sw_futInfo.WriteLine(rec.GetValAsStringByIndex(i));
                    }
                }
                sw_futInfo.Flush();

            }
            catch (System.Exception e)
            {
                LogWriteLine(e.Message + "   " + e.Source);
            }
        }
        [MTAThread]
        void StreamDataInsertedFutCommon(CP2DataStream stream, String tableName, CP2Record rec)
        {
            try
            {
                long isin_id = rec.GetValAsLong("isin_id");
                string short_isin = (string)cdFutInfo.convertIsinIdToShortIsin((int)isin_id);
                if (short_isin != null)
                {
                    bool newRow = false;

                    DataRow tempRow = futCommonTable.Rows.Find((object)short_isin);
                    if (tempRow == null)
                    {
                        newRow = true;
                        tempRow = futCommonTable.NewRow();
                    }

                    tempRow[0] = short_isin;
                    tempRow[1] = Convert.ToDouble(rec.GetValAsVariant("price"));
                    tempRow[2] = Convert.ToDouble(rec.GetValAsVariant("min_price"));
                    tempRow[3] = Convert.ToDouble(rec.GetValAsVariant("max_price"));
                    tempRow[4] = Convert.ToDateTime(rec.GetValAsString("deal_time")).ToString("HH:mm:ss");
                    tempRow[5] = Convert.ToDouble(rec.GetValAsVariant("capital"));

                    if (newRow) futCommonTable.Rows.Add(tempRow);

                   //futCommonTable.AcceptChanges();
                }
            }
            catch (System.Exception e)
            {
                LogWriteLine("event: StreamDataInsertedPart   " + e.Message);
            }
        }
        [MTAThread]
        void StreamLifeNumChanged(CP2DataStream stream, int lifeNum)
        {
            string filename = "";
            if (stream.StreamName == "FORTS_FUTTRADE_REPL")
                filename = "fut_trades.ini";
            else if (stream.StreamName == "FORTS_POS_REPL")
                filename = "pos.ini";
            else if (stream.StreamName == "FORTS_PART_REPL")
                filename = "part.ini";
            else if (stream.StreamName == "FORTS_FUTINFO_REPL")
                filename = "fut_info.ini";
            else if (stream.StreamName == "FORTS_FUTCOMMON_REPL")
                filename = "fut_common.ini";
            stream.TableSet.LifeNum = lifeNum;
            stream.TableSet.SetLifeNumToIni("sheme.ini");
            LogWriteLine("(StreamLifeNumChanged)новый номер жизния для потока " + stream.StreamName + ": " + lifeNum);
        }
        [MTAThread]
        void StreamDataDeleted(CP2DataStream stream, String tableName, Int64 Id, CP2Record rec)
        {
            LogWriteLine("(StreamDataDeleted)удаление в: " + tableName);
            LogWriteRec(rec);
        }
        [MTAThread]
        void StreamDataDatumDeleted(CP2DataStream stream, String tableName, long rev)
        {
            LogWriteLine("(StreamDataDatumDeleted)минимальный ревижен в: " + tableName + "= " + rev);
        }
        //Запись лога Плаза2
        [MTAThread]
        public void LogWriteLine(string s)
        {
            if (sw_logFile == null)
            {
                sw_logFile = new StreamWriter(pathP2Log, true, System.Text.Encoding.Unicode);
            }
            DateTime dt = DateTime.Now;
            sw_logFile.WriteLine(dt + "   " + s);
            sw_logFile.Flush();
        }
        //Запись лога стакана
        [MTAThread]
        void LogWriteGlass(string s)
        {
            if (sw_Aggregates == null)
            {
                sw_Aggregates = new StreamWriter(pathGlass, true);
            }
            sw_Aggregates.WriteLine(s);
            sw_Aggregates.Flush();
        }
        //Запись лога
        [MTAThread]
        void LogWriteRec(CP2Record rec)
        {
            if (sw_logFile == null)
            {
                sw_logFile = new StreamWriter(pathP2Log, true, System.Text.Encoding.Unicode);
            }
            uint count = rec.Count;
            for (uint i = 0; i < count; ++i)
            {
                if (i != count - 1)
                {
                    sw_logFile.Write(rec.GetValAsStringByIndex(i) + ";");
                }
                else
                {
                    sw_logFile.WriteLine(rec.GetValAsStringByIndex(i));
                }
            }
            sw_logFile.Flush();
        }
        [MTAThread]
        private void myTimerEvent(Object o, EventArgs ea)
        {
            plaza2Form.addLbls(servTime.ToString("HH:mm:ss"), streamDealsState, streamAggregatesState);
        }
        //Считывание активных ордеров
        [MTAThread]
        static public void readActiveOrder(ref long[] oAA, ref int oAAP, string comment)
        {
            oAAP = -1;
            foreach (DataRow i in ordTable.Select("comment='" + comment + "'"))
            {
                if (oAAP < oAA.Count()-1)
                {
                    oAAP++;
                    oAA[oAAP] = Convert.ToInt64(i["id_ord"]);
                }
            }
        }
        [MTAThread]
        void saveTable(string path, DataTable dt)
        {
            if (dt.Rows.Count > 0)
            {
                File.Delete(path);
                using (StreamWriter sw = File.AppendText(path))
                {
                    int count = dt.Columns.Count;
                    foreach (DataRow dr in ordTable.Select())
                    {
                        for (int i = 1; i <= count; ++i)
                        {
                            if (i != count)
                            {
                                sw.Write(Convert.ToString(dr[i - 1]) + ";");
                            }
                            else
                            {
                                sw.WriteLine(Convert.ToString(dr[i - 1]));
                            }
                        }
                    }
                }
            }
        }
        [MTAThread]
        void readTable(string path, ref DataTable dt)
        {
            if (File.Exists(path))
            {
                using (StreamReader sr = new StreamReader(path))
                {
                    int count = dt.Columns.Count;
                    while (true)
                    {
                        string s = sr.ReadLine();
                        if (s == null || s == "") break;
                        string[] sarray = s.Split(';');

                        DataRow tempRow = dt.NewRow();
                        for (int i = 0; i < count; i++)
                        {
                            tempRow[i] = Convert.ChangeType(sarray[i], dt.Columns[i].DataType);

                        }
                        dt.Rows.Add(tempRow);
                    }
                }
            }
        }
        [MTAThread]
        void saveRev(string path, long rev)
        {
            if (rev > 0)
            {
                File.Delete(path);
                using (StreamWriter sw = File.AppendText(path))
                {
                    sw.Write(rev);
                }
            }
        }
        [MTAThread]
        void readRev(string path, ref long rev)
        {
            try
            {
                if (File.Exists(path))
                {
                    using (StreamReader sr = new StreamReader(path))
                    {
                        rev = Convert.ToInt64(sr.ReadLine());
                    }
                }
                else rev = -1;
            }
            catch (Exception ex)
            {
                MessageBox.Show("Ошибка при чтении rev: " + path + ": " + ex.Message, "Ошибка");
            }
        }
    }

}
