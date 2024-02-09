using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000199 RID: 409
	[HandlerCategory("vvAverages"), HandlerName("SMMA")]
	public class SMMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CF8 RID: 3320 RVA: 0x00039050 File Offset: 0x00037250
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("smma", new string[]
			{
				this.Period.ToString(),
				this.Shift.ToString(),
				src.GetHashCode().ToString()
			}, () => SMMA.GenSMMA(src, this.Period, this.Shift));
		}

		// Token: 0x06000CF5 RID: 3317 RVA: 0x00038F58 File Offset: 0x00037158
		public static IList<double> GenSMMA(IList<double> src, int period, int shift)
		{
			IList<double> input = SMA.GenSMA(src, period);
			return SMMA._Shift(input, shift);
		}

		// Token: 0x06000CF6 RID: 3318 RVA: 0x00038F74 File Offset: 0x00037174
		public static double iSMMA(IList<double> price, IList<double> mabuf, int period, int barNum)
		{
			double result = 0.0;
			if (barNum == period)
			{
				result = SMA.iSMA(price, period, barNum);
			}
			else if (barNum > period)
			{
				double num = 0.0;
				for (int i = 0; i < period; i++)
				{
					num += price[barNum - i - 1];
				}
				result = (num - mabuf[barNum - 1] + price[barNum]) / (double)period;
			}
			return result;
		}

		// Token: 0x06000CF7 RID: 3319 RVA: 0x00038FDC File Offset: 0x000371DC
		public static IList<double> _Shift(IList<double> input, int shift)
		{
			List<double> list = new List<double>(input);
			int num = 0;
			while (list.Count > 0 && num < shift)
			{
				list.RemoveAt(list.Count - 1);
				list.Insert(0, list[1]);
				num++;
			}
			return list;
		}

		// Token: 0x1700043B RID: 1083
		public IContext Context
		{
			// Token: 0x06000CF9 RID: 3321 RVA: 0x000390CE File Offset: 0x000372CE
			get;
			// Token: 0x06000CFA RID: 3322 RVA: 0x000390D6 File Offset: 0x000372D6
			set;
		}

		// Token: 0x17000439 RID: 1081
		[HandlerParameter(true, "15", Min = "1", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000CF1 RID: 3313 RVA: 0x00038F35 File Offset: 0x00037135
			get;
			// Token: 0x06000CF2 RID: 3314 RVA: 0x00038F3D File Offset: 0x0003713D
			set;
		}

		// Token: 0x1700043A RID: 1082
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1")]
		public int Shift
		{
			// Token: 0x06000CF3 RID: 3315 RVA: 0x00038F46 File Offset: 0x00037146
			get;
			// Token: 0x06000CF4 RID: 3316 RVA: 0x00038F4E File Offset: 0x0003714E
			set;
		}
	}
}
