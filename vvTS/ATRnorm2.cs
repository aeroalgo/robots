using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200000D RID: 13
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("ATR normalized 2")]
	public class ATRnorm2 : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600006A RID: 106 RVA: 0x00003488 File Offset: 0x00001688
		public IList<double> Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> arg_20_0 = src.get_HighPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			IList<double> list;
			if (this.WATR)
			{
				list = ATR.GenWATR(src, this.Period, 0, this.Context);
			}
			else
			{
				list = ATR.GenATR(src, this.Period, 0);
			}
			for (int i = 0; i < count; i++)
			{
				array2[i] = closePrices[i] - lowPrices[i];
			}
			IList<double> list2 = SMA.GenSMA(array2, this.Period);
			for (int j = 0; j < count; j++)
			{
				array[j] = list2[j] / list[j] * 100.0;
			}
			list = array;
			if (this.smooth > 1)
			{
				list = JMA.GenJMA(array, this.smooth, this.smoothphase);
			}
			return list;
		}

		// Token: 0x17000022 RID: 34
		public IContext Context
		{
			// Token: 0x0600006B RID: 107 RVA: 0x0000357E File Offset: 0x0000177E
			get;
			// Token: 0x0600006C RID: 108 RVA: 0x00003586 File Offset: 0x00001786
			set;
		}

		// Token: 0x1700001E RID: 30
		[HandlerParameter(true, "10", Min = "5", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000062 RID: 98 RVA: 0x00003441 File Offset: 0x00001641
			get;
			// Token: 0x06000063 RID: 99 RVA: 0x00003449 File Offset: 0x00001649
			set;
		}

		// Token: 0x1700001F RID: 31
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int smooth
		{
			// Token: 0x06000064 RID: 100 RVA: 0x00003452 File Offset: 0x00001652
			get;
			// Token: 0x06000065 RID: 101 RVA: 0x0000345A File Offset: 0x0000165A
			set;
		}

		// Token: 0x17000020 RID: 32
		[HandlerParameter(true, "0", Min = "-100", Max = "100", Step = "25")]
		public int smoothphase
		{
			// Token: 0x06000066 RID: 102 RVA: 0x00003463 File Offset: 0x00001663
			get;
			// Token: 0x06000067 RID: 103 RVA: 0x0000346B File Offset: 0x0000166B
			set;
		}

		// Token: 0x17000021 RID: 33
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool WATR
		{
			// Token: 0x06000068 RID: 104 RVA: 0x00003474 File Offset: 0x00001674
			get;
			// Token: 0x06000069 RID: 105 RVA: 0x0000347C File Offset: 0x0000167C
			set;
		}
	}
}
