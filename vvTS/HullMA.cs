using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000178 RID: 376
	[HandlerCategory("vvAverages"), HandlerName("Hull's MA")]
	public class HullMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000BE3 RID: 3043 RVA: 0x00032FC8 File Offset: 0x000311C8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("HullMA", new string[]
			{
				this.Period.ToString(),
				this.Speed.ToString(),
				this.Zerolag.ToString(),
				src.GetHashCode().ToString()
			}, () => HullMA.GenZlHullMA(src, this.Period, this.Zerolag, this.Speed));
		}

		// Token: 0x06000BE2 RID: 3042 RVA: 0x00032EE4 File Offset: 0x000310E4
		public static IList<double> GenHullMA(IList<double> src, int period, double hmaspeed = 2.0)
		{
			int count = src.Count;
			int period2 = Convert.ToInt32((double)period / hmaspeed);
			Convert.ToInt32(Math.Sqrt((double)period));
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = LWMA.GenWMA(src, period2);
			IList<double> list4 = LWMA.GenWMA(src, period);
			for (int i = 0; i < count; i++)
			{
				if (i < period)
				{
					list2[i] = src[period];
				}
				else
				{
					list2[i] = 2.0 * list3[i] - list4[i];
				}
			}
			int period3 = Convert.ToInt32(Math.Sqrt((double)period));
			return LWMA.GenWMA(list2, period3);
		}

		// Token: 0x06000BE1 RID: 3041 RVA: 0x00032E84 File Offset: 0x00031084
		public static IList<double> GenZlHullMA(IList<double> src, int period, bool zerolag, double hmaspeed = 2.0)
		{
			double[] array = new double[src.Count];
			IList<double> list = HullMA.GenHullMA(src, period, hmaspeed);
			if (!zerolag)
			{
				return list;
			}
			IList<double> list2 = HullMA.GenHullMA(list, period, hmaspeed);
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = 2.0 * list[i] - list2[i];
			}
			return array;
		}

		// Token: 0x06000BE4 RID: 3044 RVA: 0x00033058 File Offset: 0x00031258
		public static double iHMA(IList<double> price, int period, int barNum, double hmaspeed = 2.0)
		{
			int num = Convert.ToInt32(Math.Sqrt((double)period));
			double[] array = new double[num];
			double result = 0.0;
			if (barNum <= period)
			{
				result = price[barNum];
			}
			else if (barNum > period)
			{
				for (int i = 0; i < num; i++)
				{
					array[num - i - 1] = 2.0 * LWMA.iLWMA(price, Convert.ToInt32((double)period / hmaspeed), barNum - i) - LWMA.iLWMA(price, period, barNum - i);
				}
				result = LWMA.iLWMA(array, num, num - 1);
			}
			return result;
		}

		// Token: 0x170003E9 RID: 1001
		public IContext Context
		{
			// Token: 0x06000BE5 RID: 3045 RVA: 0x000330DB File Offset: 0x000312DB
			get;
			// Token: 0x06000BE6 RID: 3046 RVA: 0x000330E3 File Offset: 0x000312E3
			set;
		}

		// Token: 0x170003E6 RID: 998
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000BDB RID: 3035 RVA: 0x00032E51 File Offset: 0x00031051
			get;
			// Token: 0x06000BDC RID: 3036 RVA: 0x00032E59 File Offset: 0x00031059
			set;
		}

		// Token: 0x170003E7 RID: 999
		[HandlerParameter(true, "2", Min = "0", Max = "5", Step = "0.1")]
		public double Speed
		{
			// Token: 0x06000BDD RID: 3037 RVA: 0x00032E62 File Offset: 0x00031062
			get;
			// Token: 0x06000BDE RID: 3038 RVA: 0x00032E6A File Offset: 0x0003106A
			set;
		}

		// Token: 0x170003E8 RID: 1000
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Zerolag
		{
			// Token: 0x06000BDF RID: 3039 RVA: 0x00032E73 File Offset: 0x00031073
			get;
			// Token: 0x06000BE0 RID: 3040 RVA: 0x00032E7B File Offset: 0x0003107B
			set;
		}
	}
}
