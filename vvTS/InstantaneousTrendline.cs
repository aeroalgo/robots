using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200017B RID: 379
	[HandlerCategory("vvAverages"), HandlerName("iTrend")]
	public class InstantaneousTrendline : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000BFA RID: 3066 RVA: 0x00033764 File Offset: 0x00031964
		public IList<double> Execute(ISecurity src)
		{
			return this.Context.GetData("iTrend", new string[]
			{
				this.Alpha.ToString(),
				this.Trigger.ToString(),
				src.GetHashCode().ToString()
			}, () => InstantaneousTrendline.GenITrend(src, this.Alpha, this.Trigger));
		}

		// Token: 0x06000BF7 RID: 3063 RVA: 0x0003348C File Offset: 0x0003168C
		public static IList<double> GenITrend(ISecurity src, double _Alpha, bool _Trigger)
		{
			IList<double> list = new List<double>(src.get_Bars().Count);
			IList<double> list2 = new List<double>(src.get_Bars().Count);
			list.Add(InstantaneousTrendline.GetMedPrice(src, 0));
			list.Add(InstantaneousTrendline.GetMedPrice(src, 1));
			list2.Add(2.0 * list[0] - list[0]);
			list2.Add(2.0 * list[1] - list[0]);
			for (int i = 2; i < src.get_Bars().Count; i++)
			{
				double item;
				if (i > 8)
				{
					item = (_Alpha - _Alpha * _Alpha / 4.0) * InstantaneousTrendline.GetMedPrice(src, i) + 0.5 * _Alpha * _Alpha * InstantaneousTrendline.GetMedPrice(src, i - 1) - (_Alpha - 0.75 * _Alpha * _Alpha) * InstantaneousTrendline.GetMedPrice(src, i - 2) + 2.0 * (1.0 - _Alpha) * list[i - 1] - (1.0 - _Alpha) * (1.0 - _Alpha) * list[i - 2];
				}
				else
				{
					item = (InstantaneousTrendline.GetMedPrice(src, i) + 2.0 * InstantaneousTrendline.GetMedPrice(src, i - 1) + InstantaneousTrendline.GetMedPrice(src, i - 2)) / 4.0;
				}
				list.Add(item);
				item = 2.0 * list[i] - list[i - 2];
				list2.Add(item);
			}
			if (!_Trigger)
			{
				return list;
			}
			return list2;
		}

		// Token: 0x06000BF8 RID: 3064 RVA: 0x00033624 File Offset: 0x00031824
		private static double GetMedPrice(ISecurity src, int bar)
		{
			return (src.get_HighPrices()[bar] + src.get_LowPrices()[bar]) / 2.0;
		}

		// Token: 0x06000BF9 RID: 3065 RVA: 0x0003364C File Offset: 0x0003184C
		public static double iITrend(IList<double> price, IList<double> array, int period, int barNum)
		{
			double num = 2.0 / (double)(period + 1);
			double result;
			if (barNum > 7)
			{
				result = (num - 0.25 * num * num) * price[barNum] + 0.5 * num * num * price[barNum - 1] - (num - 0.75 * num * num) * price[barNum - 2] + 2.0 * (1.0 - num) * array[barNum - 1] - (1.0 - num) * (1.0 - num) * array[barNum - 2];
			}
			else
			{
				result = (price[barNum] + 2.0 * price[barNum - 1] + price[barNum - 2]) / 4.0;
			}
			return result;
		}

		// Token: 0x170003EC RID: 1004
		[HandlerParameter(true, "0.07", Min = "0.01", Max = "1", Step = "0.01")]
		public double Alpha
		{
			// Token: 0x06000BF3 RID: 3059 RVA: 0x00033468 File Offset: 0x00031668
			get;
			// Token: 0x06000BF4 RID: 3060 RVA: 0x00033470 File Offset: 0x00031670
			set;
		}

		// Token: 0x170003EE RID: 1006
		public IContext Context
		{
			// Token: 0x06000BFB RID: 3067 RVA: 0x000337E2 File Offset: 0x000319E2
			get;
			// Token: 0x06000BFC RID: 3068 RVA: 0x000337EA File Offset: 0x000319EA
			set;
		}

		// Token: 0x170003ED RID: 1005
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Trigger
		{
			// Token: 0x06000BF5 RID: 3061 RVA: 0x00033479 File Offset: 0x00031679
			get;
			// Token: 0x06000BF6 RID: 3062 RVA: 0x00033481 File Offset: 0x00031681
			set;
		}
	}
}
