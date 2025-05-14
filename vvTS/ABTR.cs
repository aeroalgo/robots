using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200000A RID: 10
	[HandlerCategory("vvIndicators"), HandlerName("Body ATR")]
	public class ABTR : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600004D RID: 77 RVA: 0x0000316F File Offset: 0x0000136F
		public IList<double> Execute(ISecurity src)
		{
			return ABTR.GenABTR(src, this.Period, this.Smooth);
		}

		// Token: 0x0600004B RID: 75 RVA: 0x00003054 File Offset: 0x00001254
		public static IList<double> GenABTR(ISecurity sec, int period, int smooth)
		{
			int count = sec.get_Bars().Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				if (i > 0)
				{
					array[i] = ABTR.iABTR(sec, period, i);
				}
				else
				{
					array[i] = (sec.get_HighPrices()[i] - sec.get_LowPrices()[i]) / 2.0;
				}
			}
			IList<double> result = array;
			if (smooth > 0)
			{
				result = JMA.GenJMA(array, smooth, 0);
			}
			return result;
		}

		// Token: 0x0600004C RID: 76 RVA: 0x000030C8 File Offset: 0x000012C8
		public static double iABTR(ISecurity src, int period, int barNum)
		{
			if (barNum < period)
			{
				period = barNum;
			}
			List<double> list = new List<double>(period);
			for (int i = barNum - period + 1; i <= barNum; i++)
			{
				double val = Math.Max(src.get_OpenPrices()[i], src.get_ClosePrices()[i]);
				double val2 = Math.Min(src.get_OpenPrices()[i], src.get_ClosePrices()[i]);
				double val3 = src.get_ClosePrices()[i - 1];
				list.Add(Math.Max(val, val3) - Math.Min(val2, val3));
			}
			IList<double> list2 = SMA.GenSMA(list, period);
			return list2[list2.Count - 1];
		}

		// Token: 0x17000017 RID: 23
		public IContext Context
		{
			// Token: 0x0600004E RID: 78 RVA: 0x00003183 File Offset: 0x00001383
			get;
			// Token: 0x0600004F RID: 79 RVA: 0x0000318B File Offset: 0x0000138B
			set;
		}

		// Token: 0x17000015 RID: 21
		[HandlerParameter(true, "10", Min = "5", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000047 RID: 71 RVA: 0x00003031 File Offset: 0x00001231
			get;
			// Token: 0x06000048 RID: 72 RVA: 0x00003039 File Offset: 0x00001239
			set;
		}

		// Token: 0x17000016 RID: 22
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000049 RID: 73 RVA: 0x00003042 File Offset: 0x00001242
			get;
			// Token: 0x0600004A RID: 74 RVA: 0x0000304A File Offset: 0x0000124A
			set;
		}
	}
}
