using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000018 RID: 24
	[HandlerCategory("vvIndicators"), HandlerName("Chaikin's Volatility")]
	public class ChaikinsVolty : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060000C7 RID: 199 RVA: 0x000047DC File Offset: 0x000029DC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("ChVolty", new string[]
			{
				this.iPeriod.ToString(),
				this.MaPeriod.ToString(),
				this.SMAsmooth.ToString(),
				sec.get_CacheName()
			}, () => ChaikinsVolty.GenChVolty(sec, this.Context, this.iPeriod, this.MaPeriod, this.SMAsmooth));
		}

		// Token: 0x060000C6 RID: 198 RVA: 0x00004694 File Offset: 0x00002894
		public static IList<double> GenChVolty(ISecurity sec, IContext ctx, int iperiod, int maperiod, bool smasmooth)
		{
			int count = sec.get_Bars().Count;
			Math.Max(iperiod, maperiod);
			IList<double> arg_1A_0 = sec.get_ClosePrices();
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> arg_2F_0 = sec.get_OpenPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			for (int i = 0; i < count; i++)
			{
				array2[i] = highPrices[i] - lowPrices[i];
			}
			IList<double> list = smasmooth ? SMA.GenSMA(array2, maperiod) : EMA.GenEMA(array2, maperiod);
			for (int j = 1; j < iperiod; j++)
			{
				array[j] = (list[j] - list[0]) / list[0] * 100.0;
			}
			for (int k = iperiod; k < count; k++)
			{
				array[k] = (list[k] - list[k - iperiod]) / list[k - iperiod] * 100.0;
			}
			return array;
		}

		// Token: 0x17000041 RID: 65
		public IContext Context
		{
			// Token: 0x060000C8 RID: 200 RVA: 0x00004863 File Offset: 0x00002A63
			get;
			// Token: 0x060000C9 RID: 201 RVA: 0x0000486B File Offset: 0x00002A6B
			set;
		}

		// Token: 0x1700003E RID: 62
		[HandlerParameter(true, "10", Min = "5", Max = "20", Step = "1")]
		public int iPeriod
		{
			// Token: 0x060000C0 RID: 192 RVA: 0x0000465E File Offset: 0x0000285E
			get;
			// Token: 0x060000C1 RID: 193 RVA: 0x00004666 File Offset: 0x00002866
			set;
		}

		// Token: 0x1700003F RID: 63
		[HandlerParameter(true, "10", Min = "5", Max = "20", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x060000C2 RID: 194 RVA: 0x0000466F File Offset: 0x0000286F
			get;
			// Token: 0x060000C3 RID: 195 RVA: 0x00004677 File Offset: 0x00002877
			set;
		}

		// Token: 0x17000040 RID: 64
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SMAsmooth
		{
			// Token: 0x060000C4 RID: 196 RVA: 0x00004680 File Offset: 0x00002880
			get;
			// Token: 0x060000C5 RID: 197 RVA: 0x00004688 File Offset: 0x00002888
			set;
		}
	}
}
