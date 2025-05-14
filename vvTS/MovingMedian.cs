using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200018B RID: 395
	[HandlerCategory("vvAverages"), HandlerName("Moving Median")]
	public class MovingMedian : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C7A RID: 3194 RVA: 0x00036140 File Offset: 0x00034340
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("MovingMedian", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => MovingMedian.GenMovMedian(src, this.Period));
		}

		// Token: 0x06000C78 RID: 3192 RVA: 0x00036058 File Offset: 0x00034258
		public static IList<double> GenMovMedian(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = MovingMedian.iMedian(src, period, i);
				}
			}
			return array;
		}

		// Token: 0x06000C79 RID: 3193 RVA: 0x000360A0 File Offset: 0x000342A0
		public static double iMedian(IList<double> price, int period, int barNum)
		{
			List<double> list = new List<double>();
			for (int i = 0; i < period; i++)
			{
				list.Add(price[barNum - i]);
			}
			list.Sort();
			int num = Convert.ToInt32(Math.Round(Convert.ToDouble((period - 1) / 2)));
			double result;
			if (period % 2 > 0)
			{
				result = list[num];
			}
			else
			{
				result = 0.5 * (list[num] + list[num + 1]);
			}
			return result;
		}

		// Token: 0x17000414 RID: 1044
		public IContext Context
		{
			// Token: 0x06000C7B RID: 3195 RVA: 0x000361AC File Offset: 0x000343AC
			get;
			// Token: 0x06000C7C RID: 3196 RVA: 0x000361B4 File Offset: 0x000343B4
			set;
		}

		// Token: 0x17000413 RID: 1043
		[HandlerParameter(true, "15", Min = "7", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000C76 RID: 3190 RVA: 0x00036045 File Offset: 0x00034245
			get;
			// Token: 0x06000C77 RID: 3191 RVA: 0x0003604D File Offset: 0x0003424D
			set;
		}
	}
}
