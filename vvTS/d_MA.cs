using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000162 RID: 354
	[HandlerCategory("vvAverages"), HandlerName("dMA trend adaptive")]
	public class d_MA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B35 RID: 2869 RVA: 0x0002E0C0 File Offset: 0x0002C2C0
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("dMA", new string[]
			{
				this.Period.ToString(),
				this.Method.ToString(),
				src.GetHashCode().ToString()
			}, () => d_MA.GenD_MA(src, this.Context, this.Period, this.Method));
		}

		// Token: 0x06000B34 RID: 2868 RVA: 0x0002DF1C File Offset: 0x0002C11C
		public static IList<double> GenD_MA(IList<double> src, IContext Context, int period, int method = 0)
		{
			int count = src.Count;
			int num = period;
			double[] array = new double[count];
			period = Math.Max(2, period);
			for (int i = 0; i < count; i++)
			{
				if (i < num)
				{
					array[i] = src[i];
				}
				else
				{
					double num2 = vvSeries.iMA(src, array, method, num, i - 1, 1.0, 0.0);
					double num3 = vvSeries.iMA(src, array, method, num, i - 2, 1.0, 0.0);
					array[i] = vvSeries.iMA(src, array, method, num, i, 1.0, 0.0);
					if (src[i - 1] > src[i - 2] && src[i - 1] - num2 > src[i - 2] - num3)
					{
						num--;
						if (num < 2)
						{
							num = 2;
						}
					}
					if (src[i - 1] < src[i - 2] && src[i - 1] - num2 < src[i - 2] - num3)
					{
						num--;
						if (num < 2)
						{
							num = 2;
						}
					}
					if (src[i - 1] > num2 && src[i - 2] < num3)
					{
						num = period;
					}
					if (src[i - 1] < num2 && src[i - 2] > num3)
					{
						num = period;
					}
				}
			}
			return array;
		}

		// Token: 0x170003B2 RID: 946
		public IContext Context
		{
			// Token: 0x06000B36 RID: 2870 RVA: 0x0002E13E File Offset: 0x0002C33E
			get;
			// Token: 0x06000B37 RID: 2871 RVA: 0x0002E146 File Offset: 0x0002C346
			set;
		}

		// Token: 0x170003B1 RID: 945
		[HandlerParameter(true, "0", Min = "0", Max = "5", Step = "1")]
		public int Method
		{
			// Token: 0x06000B32 RID: 2866 RVA: 0x0002DF0A File Offset: 0x0002C10A
			get;
			// Token: 0x06000B33 RID: 2867 RVA: 0x0002DF12 File Offset: 0x0002C112
			set;
		}

		// Token: 0x170003B0 RID: 944
		[HandlerParameter(true, "20", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000B30 RID: 2864 RVA: 0x0002DEF9 File Offset: 0x0002C0F9
			get;
			// Token: 0x06000B31 RID: 2865 RVA: 0x0002DF01 File Offset: 0x0002C101
			set;
		}
	}
}
