using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000153 RID: 339
	[HandlerCategory("vvAverages"), HandlerName("AllAverages")]
	public class AllAverages : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000AA5 RID: 2725 RVA: 0x0002C0C7 File Offset: 0x0002A2C7
		public IList<double> Execute(IList<double> src)
		{
			return AllAverages.Gen_mMA(src, this.Context, this.MAType, this.MaPeriod, this.MaPeriod2, this.MaParam1, this.MaParam2);
		}

		// Token: 0x06000AA4 RID: 2724 RVA: 0x0002BE50 File Offset: 0x0002A050
		public static IList<double> Gen_mMA(IList<double> src, IContext ctx, int matype, int maperiod, int maperiod2, double maparam1, double maparam2)
		{
			IList<double> result;
			switch (matype)
			{
			case 1:
				result = SMMA.GenSMMA(src, maperiod, maperiod2);
				return result;
			case 2:
				result = EMA.GenEMA(src, maperiod);
				return result;
			case 3:
				result = LWMA.GenWMA(src, maperiod);
				return result;
			case 4:
				result = JMA.GenJMA(src, maperiod, Convert.ToInt32(maparam1));
				return result;
			case 5:
				result = HullMA.GenHullMA(src, maperiod, 2.0);
				return result;
			case 6:
				result = AMA.GenAMA(src, maperiod);
				return result;
			case 7:
				result = LRMA.GenLRMA(src, maperiod);
				return result;
			case 8:
				result = ALMA.GenALMA(src, maperiod, maparam1, maparam2);
				return result;
			case 9:
				result = Bezier.GenBezier(src, maperiod, maparam1);
				return result;
			case 10:
				result = DEMA.GenDEMA(src, ctx, maperiod);
				return result;
			case 11:
				result = REMA.GenREMA(src, maperiod, maparam1);
				return result;
			case 12:
				result = GaussFilter.GenGaussFilter(src, maperiod, Convert.ToInt32((maparam1 == 0.0) ? 2.0 : maparam1));
				return result;
			case 13:
				result = KalmanFilter.GenKF(src, (maparam1 <= 0.0) ? 1.0 : maparam1, (maparam2 <= 0.0) ? 1.0 : maparam2);
				return result;
			case 14:
				result = MEMA.GenMEMA(src, (maparam1 <= 0.0) ? 0.1 : maparam1);
				return result;
			case 15:
				result = NMA.GenNMA(src, maperiod, Convert.ToInt32((maparam1 <= 0.0) ? 1.0 : maparam1));
				return result;
			case 16:
				result = NonLagMA.GenNonLagMA(src, maperiod, maparam1, maparam2);
				return result;
			case 17:
				result = QuickMA.GenQuickMA(src, maperiod);
				return result;
			case 18:
				result = SineWMA.GenSineWMA(src, maperiod);
				return result;
			case 19:
				result = TEMA.GenTEMA(src, maperiod);
				return result;
			case 20:
				result = T3Tilson.GenT3(src, maperiod, maparam1, true);
				return result;
			case 21:
				result = VIDYA.GenVIDYA(src, maperiod, Convert.ToInt32((maparam1 == 0.0) ? 30.0 : maparam1));
				return result;
			case 22:
				result = WilderMA.GenWilderMA(src, maperiod);
				return result;
			case 23:
				result = ZLEMA.GenZLEMA(src, maperiod);
				return result;
			case 25:
				result = MaRsiAdaptive.GenMaRsiAdaptive(src, maperiod, maparam1);
				return result;
			}
			result = SMA.GenSMA(src, maperiod);
			return result;
		}

		// Token: 0x17000387 RID: 903
		public IContext Context
		{
			// Token: 0x06000AA6 RID: 2726 RVA: 0x0002C0F3 File Offset: 0x0002A2F3
			get;
			// Token: 0x06000AA7 RID: 2727 RVA: 0x0002C0FB File Offset: 0x0002A2FB
			set;
		}

		// Token: 0x17000385 RID: 901
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1")]
		public double MaParam1
		{
			// Token: 0x06000AA0 RID: 2720 RVA: 0x0002BE2C File Offset: 0x0002A02C
			get;
			// Token: 0x06000AA1 RID: 2721 RVA: 0x0002BE34 File Offset: 0x0002A034
			set;
		}

		// Token: 0x17000386 RID: 902
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1")]
		public double MaParam2
		{
			// Token: 0x06000AA2 RID: 2722 RVA: 0x0002BE3D File Offset: 0x0002A03D
			get;
			// Token: 0x06000AA3 RID: 2723 RVA: 0x0002BE45 File Offset: 0x0002A045
			set;
		}

		// Token: 0x17000383 RID: 899
		[HandlerParameter(true, "10", Min = "1", Max = "20", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x06000A9C RID: 2716 RVA: 0x0002BE0A File Offset: 0x0002A00A
			get;
			// Token: 0x06000A9D RID: 2717 RVA: 0x0002BE12 File Offset: 0x0002A012
			set;
		}

		// Token: 0x17000384 RID: 900
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int MaPeriod2
		{
			// Token: 0x06000A9E RID: 2718 RVA: 0x0002BE1B File Offset: 0x0002A01B
			get;
			// Token: 0x06000A9F RID: 2719 RVA: 0x0002BE23 File Offset: 0x0002A023
			set;
		}

		// Token: 0x17000382 RID: 898
		[HandlerParameter(true, "2", Min = "0", Max = "25", Step = "1")]
		public int MAType
		{
			// Token: 0x06000A9A RID: 2714 RVA: 0x0002BDF9 File Offset: 0x00029FF9
			get;
			// Token: 0x06000A9B RID: 2715 RVA: 0x0002BE01 File Offset: 0x0002A001
			set;
		}
	}
}
