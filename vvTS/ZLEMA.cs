using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001AC RID: 428
	[HandlerCategory("vvAverages"), HandlerName("ZLEMA")]
	public class ZLEMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D94 RID: 3476 RVA: 0x0003B5D4 File Offset: 0x000397D4
		public IList<double> Execute(IList<double> src)
		{
			this.Context.GetData("zlema", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => ZLEMA.GenZLEMA(src, this.Period));
			return ZLEMA.GenZLEMA(src, this.Period);
		}

		// Token: 0x06000D92 RID: 3474 RVA: 0x0003B4D8 File Offset: 0x000396D8
		public static IList<double> GenZLEMA(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			double num = 2.0 / ((double)period + 1.0);
			int num2 = (period - 1) / 2;
			for (int i = num2; i < src.Count; i++)
			{
				array[i] = array[i - 1] + num * (src[i] + (src[i] - src[i - num2]) - array[i - 1]);
			}
			return array;
		}

		// Token: 0x06000D93 RID: 3475 RVA: 0x0003B558 File Offset: 0x00039758
		public static double iZLEMA(IList<double> P, double prevZlema, int period, int barNum)
		{
			double num = 2.0 / ((double)period + 1.0);
			int num2 = (period - 1) / 2;
			if (num2 > barNum)
			{
				num2 = barNum;
			}
			return prevZlema + num * (P[barNum] + (P[barNum] - P[barNum - num2]) - prevZlema);
		}

		// Token: 0x17000469 RID: 1129
		public IContext Context
		{
			// Token: 0x06000D95 RID: 3477 RVA: 0x0003B652 File Offset: 0x00039852
			get;
			// Token: 0x06000D96 RID: 3478 RVA: 0x0003B65A File Offset: 0x0003985A
			set;
		}

		// Token: 0x17000468 RID: 1128
		[HandlerParameter(true, "20", Min = "1", Max = "100", Step = "1")]
		public int Period
		{
			// Token: 0x06000D90 RID: 3472 RVA: 0x0003B4C6 File Offset: 0x000396C6
			get;
			// Token: 0x06000D91 RID: 3473 RVA: 0x0003B4CE File Offset: 0x000396CE
			set;
		}
	}
}
