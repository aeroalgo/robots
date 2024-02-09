using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200005D RID: 93
	[HandlerCategory("vvIndicators"), HandlerName("SuperTrend")]
	public class SuperTrend : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000355 RID: 853 RVA: 0x00013064 File Offset: 0x00011264
		public IList<double> Execute(ISecurity src)
		{
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> list = ATR.GenWATR(src, this.ATRPeriod, 0, this.Context);
			IList<double> data = this.Context.GetData("MedianPrice", new string[]
			{
				src.get_CacheName()
			}, () => Series.MedianPrice(src.get_Bars()));
			double[] array = new double[closePrices.Count];
			for (int i = 2; i < closePrices.Count; i++)
			{
				double num = data[i] + list[i] * this.ATRmultiplier;
				double num2 = data[i] - list[i] * this.ATRmultiplier;
				if (closePrices[i] >= array[i - 1])
				{
					if (closePrices[i - 1] < array[i - 1])
					{
						array[i] = num2;
					}
					else
					{
						array[i] = ((num2 > array[i - 1]) ? num2 : array[i - 1]);
					}
				}
				else if (closePrices[i] < array[i - 1])
				{
					if (closePrices[i - 1] > array[i - 1])
					{
						array[i] = num;
					}
					else
					{
						array[i] = ((num < array[i - 1]) ? num : array[i - 1]);
					}
				}
			}
			return array;
		}

		// Token: 0x1700011D RID: 285
		[HandlerParameter(true, "2.0", Min = "2.0", Max = "15", Step = "0.1")]
		public double ATRmultiplier
		{
			// Token: 0x06000351 RID: 849 RVA: 0x00013027 File Offset: 0x00011227
			get;
			// Token: 0x06000352 RID: 850 RVA: 0x0001302F File Offset: 0x0001122F
			set;
		}

		// Token: 0x1700011E RID: 286
		[HandlerParameter(true, "7", Min = "1", Max = "30", Step = "1")]
		public int ATRPeriod
		{
			// Token: 0x06000353 RID: 851 RVA: 0x00013038 File Offset: 0x00011238
			get;
			// Token: 0x06000354 RID: 852 RVA: 0x00013040 File Offset: 0x00011240
			set;
		}

		// Token: 0x1700011F RID: 287
		public IContext Context
		{
			// Token: 0x06000356 RID: 854 RVA: 0x000131C1 File Offset: 0x000113C1
			get;
			// Token: 0x06000357 RID: 855 RVA: 0x000131C9 File Offset: 0x000113C9
			set;
		}
	}
}
