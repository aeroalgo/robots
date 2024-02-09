using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000157 RID: 343
	[HandlerCategory("vvAverages"), HandlerName("AMA")]
	public class AMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000AC5 RID: 2757 RVA: 0x0002C8D4 File Offset: 0x0002AAD4
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("ama", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => AMA.GenAMA(src, this.Period));
		}

		// Token: 0x06000AC4 RID: 2756 RVA: 0x0002C79C File Offset: 0x0002A99C
		public static IList<double> GenAMA(IList<double> src, int period)
		{
			int count = src.Count;
			double[] array = new double[count];
			if (period > 0 && count > 0)
			{
				for (int i = 0; i <= period * 2; i++)
				{
					array[i] = src[i];
				}
				double num = (count < period) ? 0.0 : src[period + 1];
				for (int j = period + 2; j < count; j++)
				{
					double num2 = Math.Abs(src[j] - src[j - period]);
					double num3 = 1E-09;
					for (int k = 0; k < period; k++)
					{
						num3 += Math.Abs(src[j - k] - src[j - k - 1]);
					}
					double num4 = num2 / num3;
					double x = num4 * 0.60215 + 0.06452;
					double num5 = Math.Pow(x, 2.0);
					double num6 = num + num5 * (src[j] - num);
					array[j] = num6;
					num = num6;
				}
			}
			return array;
		}

		// Token: 0x17000391 RID: 913
		public IContext Context
		{
			// Token: 0x06000AC6 RID: 2758 RVA: 0x0002C940 File Offset: 0x0002AB40
			get;
			// Token: 0x06000AC7 RID: 2759 RVA: 0x0002C948 File Offset: 0x0002AB48
			set;
		}

		// Token: 0x17000390 RID: 912
		[HandlerParameter(true, "20", Min = "10", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000AC2 RID: 2754 RVA: 0x0002C789 File Offset: 0x0002A989
			get;
			// Token: 0x06000AC3 RID: 2755 RVA: 0x0002C791 File Offset: 0x0002A991
			set;
		}
	}
}
