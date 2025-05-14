using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000182 RID: 386
	[HandlerCategory("vvAverages"), HandlerName("LWMA")]
	public class LWMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C39 RID: 3129 RVA: 0x000350D4 File Offset: 0x000332D4
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("lwma", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => LWMA.GenWMA(src, this.Period));
		}

		// Token: 0x06000C35 RID: 3125 RVA: 0x00034EAC File Offset: 0x000330AC
		public static IList<double> GenWMA(IList<double> src, int period)
		{
			if (src.Count < period || src.Count < 2)
			{
				return null;
			}
			double[] array = new double[src.Count];
			double num = 0.0;
			int num2 = 0;
			for (int i = 0; i < period; i++)
			{
				num2 = num2 + i + 1;
			}
			for (int j = period; j < src.Count; j++)
			{
				int num3 = 1;
				for (int k = j - period + 1; k <= j; k++)
				{
					num += (double)num3 * src[k];
					num3++;
				}
				array[j] = num / (double)num2;
				num = 0.0;
			}
			return array;
		}

		// Token: 0x06000C36 RID: 3126 RVA: 0x00034F50 File Offset: 0x00033150
		public static IList<double> GenWMA_oldmethod(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			double num = 0.0;
			int num2 = 0;
			for (int i = 0; i < period; i++)
			{
				num2 = num2 + i + 1;
			}
			for (int j = period; j < src.Count; j++)
			{
				int num3 = 1;
				for (int k = j - period + 1; k <= j; k++)
				{
					num += (double)num3 * src[k];
					num3++;
				}
				array[j] = num / (double)num2;
				num = 0.0;
			}
			return array;
		}

		// Token: 0x06000C38 RID: 3128 RVA: 0x00035040 File Offset: 0x00033240
		public static double iLWMA(IList<double> price, int period, int barNum)
		{
			if (barNum < period)
			{
				period = barNum;
			}
			double num = 0.0;
			double num2 = 0.0;
			for (int i = 0; i < period; i++)
			{
				num2 += (double)(period - i);
				num += price[barNum - i] * (double)(period - i);
			}
			double result;
			if (num2 > 0.0)
			{
				result = num / num2;
			}
			else
			{
				result = 0.0;
			}
			return result;
		}

		// Token: 0x06000C37 RID: 3127 RVA: 0x00034FE0 File Offset: 0x000331E0
		public static double iWMA(IList<double> src, int curbar, int period)
		{
			if (curbar < period)
			{
				period = curbar;
			}
			double num = 0.0;
			int num2 = 0;
			for (int i = 0; i < period; i++)
			{
				num2 = num2 + i + 1;
			}
			int num3 = 1;
			for (int j = curbar - period + 1; j <= curbar; j++)
			{
				num += (double)num3 * src[j];
				num3++;
			}
			return num / (double)num2;
		}

		// Token: 0x17000401 RID: 1025
		public IContext Context
		{
			// Token: 0x06000C3A RID: 3130 RVA: 0x00035140 File Offset: 0x00033340
			get;
			// Token: 0x06000C3B RID: 3131 RVA: 0x00035148 File Offset: 0x00033348
			set;
		}

		// Token: 0x17000400 RID: 1024
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000C33 RID: 3123 RVA: 0x00034E98 File Offset: 0x00033098
			get;
			// Token: 0x06000C34 RID: 3124 RVA: 0x00034EA0 File Offset: 0x000330A0
			set;
		}
	}
}
